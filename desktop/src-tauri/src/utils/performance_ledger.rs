use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};

use serde_json::json;

const RUST_LEDGER_FILE: &str = "aether-rust-performance-ledger.jsonl";
const SLOW_RUST_TIMING_THRESHOLD: Duration = Duration::from_millis(150);

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
        "details": details,
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

    let path = std::env::temp_dir().join(RUST_LEDGER_FILE);
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let _ = writeln!(file, "{}", entry);
}

pub fn rust_ledger_path() -> std::path::PathBuf {
    std::env::temp_dir().join(RUST_LEDGER_FILE)
}
