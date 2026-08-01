use serde_json::Value;

use crate::error::Result;
use crate::utils::performance_ledger::{
    export_debug_logs as write_debug_log_export, DebugLogExportResult,
};

#[tauri::command]
pub async fn export_debug_logs(
    frontend_entries: Option<Vec<Value>>,
) -> Result<DebugLogExportResult> {
    write_debug_log_export(frontend_entries.unwrap_or_default()).map_err(Into::into)
}
