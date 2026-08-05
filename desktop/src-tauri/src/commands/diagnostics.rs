use serde_json::Value;

use crate::error::Result;
use crate::utils::performance_ledger::{
    ensure_error_ledger_file, export_debug_logs as write_debug_log_export, DebugLogExportResult,
};

#[tauri::command]
pub async fn export_debug_logs(
    frontend_entries: Option<Vec<Value>>,
) -> Result<DebugLogExportResult> {
    write_debug_log_export(frontend_entries.unwrap_or_default()).map_err(Into::into)
}

#[tauri::command]
pub async fn get_error_log_path() -> Result<String> {
    ensure_error_ledger_file()
        .map(|path| path.display().to_string())
        .map_err(Into::into)
}
