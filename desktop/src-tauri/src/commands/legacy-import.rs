use crate::db::{connection, legacy_import, SearchDocumentRepository};
use crate::error::Result;
use tauri::State;

#[tauri::command]
pub async fn preview_legacy_database(
    state: State<'_, crate::DbState>,
    source_path: String,
) -> Result<legacy_import::LegacyImportPreview> {
    let _guard = connection::with_db_access(&state).await;
    legacy_import::preview_legacy_database(&source_path).await
}

#[tauri::command]
pub async fn import_legacy_database(
    state: State<'_, crate::DbState>,
    source_path: String,
) -> Result<legacy_import::LegacyImportCounts> {
    let _guard = connection::with_db_access(&state).await;
    let database = connection::get_database(&state);
    let counts = legacy_import::import_legacy_database(&database, &source_path).await?;
    SearchDocumentRepository::new(database)
        .reindex_all()
        .await?;
    Ok(counts)
}
