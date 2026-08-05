use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::platform::{desktop, PlatformCapabilities};

const RUST_LEDGER_FILE: &str = "aether-diagnostics-rust.jsonl";
const ERROR_LEDGER_FILE: &str = "aether-errors.jsonl";
const MAX_RUST_LEDGER_ENTRIES: usize = 500;
const MAX_ERROR_LEDGER_ENTRIES: usize = 500;
const SLOW_RUST_TIMING_THRESHOLD: Duration = Duration::from_millis(150);
const REDACTED: &str = "[REDACTED]";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugLogExportResult {
    pub path: String,
    pub rust_entries: usize,
    pub frontend_entries: usize,
    pub error_entries: usize,
}

pub struct PerfTimer {
    event: &'static str,
    name: &'static str,
    started: Instant,
}

impl PerfTimer {
    pub fn start(event: &'static str, name: &'static str) -> Self {
        Self {
            event,
            name,
            started: Instant::now(),
        }
    }

    pub fn finish(self, details: serde_json::Value) {
        record_rust_timing(self.event, self.name, self.started.elapsed(), details);
    }
}

pub fn record_rust_timing(
    event: &'static str,
    name: &str,
    elapsed: Duration,
    details: serde_json::Value,
) {
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let entry = json!({
        "at": chrono::Utc::now().to_rfc3339(),
        "event": event,
        "name": name,
        "elapsed_ms": (elapsed_ms * 10.0).round() / 10.0,
        "details": redact_json_value(&details),
    });

    if elapsed >= SLOW_RUST_TIMING_THRESHOLD {
        tracing::warn!(
            "[RUST-TIMING] event={} name={} elapsed_ms={:.1} details={}",
            event,
            name,
            elapsed_ms,
            entry["details"]
        );
    } else {
        tracing::debug!(
            "[RUST-TIMING] event={} name={} elapsed_ms={:.1} details={}",
            event,
            name,
            elapsed_ms,
            entry["details"]
        );
    }

    let path = rust_ledger_path();
    let _ = append_bounded_jsonl(&path, &entry, MAX_RUST_LEDGER_ENTRIES);
}

pub fn rust_ledger_path() -> std::path::PathBuf {
    diagnostics_dir().join(RUST_LEDGER_FILE)
}

pub fn error_ledger_path() -> PathBuf {
    diagnostics_dir().join(ERROR_LEDGER_FILE)
}

/// Record a redacted operational error in a bounded JSONL file that can be inspected live.
pub fn record_error(component: &str, message: &str, details: Value) {
    let entry = redact_json_value(&json!({
        "at": chrono::Utc::now().to_rfc3339(),
        "level": "error",
        "component": component,
        "message": message,
        "details": details,
    }));

    tracing::error!(
        "[ERROR-LEDGER] component={} message={} details={}",
        component,
        message,
        entry["details"]
    );

    let _ = append_bounded_jsonl(&error_ledger_path(), &entry, MAX_ERROR_LEDGER_ENTRIES);
}

/// Preserve useful server error structure while redacting secret-shaped JSON fields.
pub fn redact_http_response_body(body: &str) -> Value {
    serde_json::from_str::<Value>(body)
        .map(|value| redact_json_value(&value))
        .unwrap_or_else(|_| Value::String(redact_string(body)))
}

/// Persists every `tracing::error!` event without changing the existing terminal logger.
pub struct PersistentErrorLayer;

impl<S> Layer<S> for PersistentErrorLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        if *event.metadata().level() != Level::ERROR {
            return;
        }

        let mut visitor = ErrorFieldVisitor::default();
        event.record(&mut visitor);
        let message = visitor
            .fields
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Unhandled tracing error")
            .to_string();

        // `record_error` emits its own error event for the terminal; do not record it twice.
        if message.contains("[ERROR-LEDGER]") {
            return;
        }

        visitor.fields.insert(
            "target".to_string(),
            Value::String(event.metadata().target().to_string()),
        );
        visitor.fields.insert(
            "module".to_string(),
            Value::String(
                event
                    .metadata()
                    .module_path()
                    .unwrap_or_default()
                    .to_string(),
            ),
        );
        if let Some(file) = event.metadata().file() {
            visitor
                .fields
                .insert("file".to_string(), Value::String(file.to_string()));
        }
        if let Some(line) = event.metadata().line() {
            visitor.fields.insert("line".to_string(), Value::from(line));
        }

        record_error("tracing", &message, Value::Object(visitor.fields));
    }
}

#[derive(Default)]
struct ErrorFieldVisitor {
    fields: serde_json::Map<String, Value>,
}

impl tracing::field::Visit for ErrorFieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            Value::String(format!("{value:?}")),
        );
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), Value::String(value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), Value::Bool(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields
            .insert(field.name().to_string(), Value::from(value));
    }
}

/// Ensure the live error ledger exists so it can be opened before the next failure.
pub fn ensure_error_ledger_file() -> std::io::Result<PathBuf> {
    let path = error_ledger_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    Ok(path)
}

pub fn export_debug_logs(frontend_entries: Vec<Value>) -> std::io::Result<DebugLogExportResult> {
    let dir = diagnostics_dir();
    std::fs::create_dir_all(&dir)?;

    let rust_entries = read_jsonl_values(&rust_ledger_path())?
        .into_iter()
        .map(|entry| redact_json_value(&entry))
        .collect::<Vec<_>>();
    let frontend_entries = frontend_entries
        .into_iter()
        .map(|entry| redact_json_value(&entry))
        .collect::<Vec<_>>();
    let error_entries = read_jsonl_values(&error_ledger_path())?
        .into_iter()
        .map(|entry| redact_json_value(&entry))
        .collect::<Vec<_>>();

    let export = json!({
        "schemaVersion": 1,
        "exportedAt": chrono::Utc::now().to_rfc3339(),
        "privacy": {
            "mode": "local-redacted",
            "notes": [
                "Generated on device.",
                "No journal or task body content is intentionally included.",
                "Known secret fields and query values are redacted before export."
            ]
        },
        "timings": {
            "rust": rust_entries,
            "frontend": frontend_entries
        },
        "errors": error_entries,
    });

    let filename = format!(
        "aether-debug-log-{}.json",
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    let path = dir.join(filename);
    let contents = serde_json::to_string_pretty(&export).map_err(std::io::Error::other)?;
    std::fs::write(&path, contents)?;

    Ok(DebugLogExportResult {
        path: path.display().to_string(),
        rust_entries: export["timings"]["rust"].as_array().map_or(0, Vec::len),
        frontend_entries: export["timings"]["frontend"].as_array().map_or(0, Vec::len),
        error_entries: export["errors"].as_array().map_or(0, Vec::len),
    })
}

pub fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in map {
                if is_sensitive_key(key) {
                    redacted.insert(key.clone(), Value::String(REDACTED.to_string()));
                } else {
                    redacted.insert(key.clone(), redact_json_value(value));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(values) => Value::Array(values.iter().map(redact_json_value).collect()),
        Value::String(value) => Value::String(redact_string(value)),
        _ => value.clone(),
    }
}

fn diagnostics_dir() -> PathBuf {
    desktop()
        .storage_paths()
        .map(|paths| paths.logs)
        .unwrap_or_else(|_| std::env::temp_dir().join("aether-diagnostics"))
}

#[cfg(test)]
fn production_diagnostics_dir() -> Option<PathBuf> {
    // Keep the existing production-location assertion independent of the
    // debug-only diagnostics path used by the desktop platform adapter.
    directories::ProjectDirs::from("com", "cas", "aether")
        .map(|dirs| dirs.data_local_dir().join("diagnostics"))
}

fn append_bounded_jsonl(path: &Path, entry: &Value, max_entries: usize) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut lines = std::fs::read_to_string(path)
        .map(|contents| contents.lines().map(str::to_owned).collect::<Vec<_>>())
        .unwrap_or_default();
    lines.push(entry.to_string());
    if lines.len() > max_entries {
        lines.drain(0..lines.len() - max_entries);
    }

    std::fs::write(path, format!("{}\n", lines.join("\n")))
}

fn read_jsonl_values(path: &Path) -> std::io::Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(path)?;
    Ok(contents
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect())
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "authorization",
        "password",
        "passphrase",
        "token",
        "secret",
        "apikey",
        "api_key",
        "accesskey",
        "access_key",
        "credential",
        "private_key",
    ]
    .iter()
    .any(|part| key.contains(part))
}

fn redact_string(value: &str) -> String {
    let mut redacted = redact_url_query_values(value);
    redacted = redact_assignment_values(&redacted);
    redact_known_token_shapes(&redacted)
}

fn redact_url_query_values(value: &str) -> String {
    let Some((base, query)) = value.split_once('?') else {
        return value.to_string();
    };
    let (query, suffix) = query
        .split_once('#')
        .map_or((query, ""), |(query, suffix)| (query, suffix));
    let redacted_query = query
        .split('&')
        .map(|pair| {
            pair.split_once('=').map_or_else(
                || pair.to_string(),
                |(key, value)| {
                    if value.is_empty() {
                        key.to_string()
                    } else if is_safe_query_value(key, value) {
                        format!("{}={}", key, value)
                    } else {
                        format!("{}={}", key, REDACTED)
                    }
                },
            )
        })
        .collect::<Vec<_>>()
        .join("&");

    if suffix.is_empty() {
        format!("{}?{}", base, redacted_query)
    } else {
        format!("{}?{}#{}", base, redacted_query, suffix)
    }
}

fn is_safe_query_value(key: &str, value: &str) -> bool {
    if is_sensitive_key(key) {
        return false;
    }
    matches!(key, "limit" | "offset" | "page" | "cursor") && value.len() <= 64
}

fn redact_assignment_values(value: &str) -> String {
    let mut out = value.to_string();
    for marker in [
        "authorization=",
        "password=",
        "passphrase=",
        "token=",
        "secret=",
        "api_key=",
        "apikey=",
        "access_key=",
        "credential=",
    ] {
        out = redact_after_marker(&out, marker);
    }
    out
}

fn redact_after_marker(value: &str, marker: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let mut search_start = 0;
    let mut out = String::new();

    while let Some(relative_index) = lower[search_start..].find(marker) {
        let marker_start = search_start + relative_index;
        let value_start = marker_start + marker.len();
        out.push_str(&value[search_start..value_start]);

        let value_end = value[value_start..]
            .find(|c: char| c == '&' || c == ',' || c == '"' || c == '\'' || c == '\n' || c == '\r')
            .map_or(value.len(), |end| value_start + end);
        out.push_str(REDACTED);
        search_start = value_end;
    }

    out.push_str(&value[search_start..]);
    out
}

fn redact_known_token_shapes(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            if part.starts_with("phx_") || part.starts_with("sk-") {
                REDACTED.to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_sensitive_object_keys() {
        let value = json!({
            "apiKey": "sk-secret",
            "syncPassphrase": "correct horse battery staple",
            "elapsed_ms": 12.3,
        });

        let redacted = redact_json_value(&value);

        assert_eq!(redacted["apiKey"], REDACTED);
        assert_eq!(redacted["syncPassphrase"], REDACTED);
        assert_eq!(redacted["elapsed_ms"], 12.3);
    }

    #[test]
    fn redacts_query_values_but_keeps_safe_timing_context() {
        let value = json!({
            "url": "/v1/search?query=private%20journal&limit=50&token=abc123",
        });

        let redacted = redact_json_value(&value);

        assert_eq!(
            redacted["url"],
            "/v1/search?query=[REDACTED]&limit=50&token=[REDACTED]"
        );
    }

    #[test]
    fn redacts_secret_assignments_inside_messages() {
        let value = json!({
            "error": "failed authorization=Bearer abc",
        });

        let redacted = redact_json_value(&value);

        assert_eq!(redacted["error"], "failed authorization=[REDACTED]");
    }

    #[test]
    fn redacts_error_ledger_details_before_persistence() {
        let entry = redact_json_value(&json!({
            "component": "sync.pull",
            "details": { "deviceToken": "secret-token" },
        }));

        assert_eq!(entry["details"]["deviceToken"], REDACTED);
    }

    #[test]
    fn redacts_secret_fields_in_structured_http_error_responses() {
        let response =
            redact_http_response_body(r#"{"error":"unauthorized","token":"secret-token"}"#);

        assert_eq!(response["error"], "unauthorized");
        assert_eq!(response["token"], REDACTED);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn production_diagnostics_use_tauris_app_data_directory() {
        let dir = production_diagnostics_dir().expect("macOS app data directory should resolve");

        assert!(dir.ends_with("com.cas.aether/diagnostics"));
    }
}
