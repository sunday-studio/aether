use crate::db::ActivityRepository;
use crate::error::Result;
use libsql::Database;
use std::sync::Arc;

/// Log an activity to the database
pub async fn log_activity(
    database: Arc<Database>,
    action_type: String,
    entity_type: String,
    entity_id: String,
    metadata: Option<serde_json::Value>,
) -> Result<()> {
    let repo = ActivityRepository::new(database);
    repo.create(action_type.clone(), entity_type.clone(), entity_id.clone(), metadata).await?;
    Ok(())
}

/// Helper to log a create action
pub async fn log_create(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(database, "create".to_string(), entity_type, entity_id, None).await
}

/// Helper to log an update action
pub async fn log_update(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(database, "update".to_string(), entity_type, entity_id, None).await
}

/// Helper to log a delete action
pub async fn log_delete(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(database, "delete".to_string(), entity_type, entity_id, None).await
}

/// Helper to log a complete action (for tasks/subtasks)
pub async fn log_complete(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(database, "complete".to_string(), entity_type, entity_id, None).await
}

/// Helper to log tag operations
pub async fn log_tag_operation(
    database: Arc<Database>,
    action: &str, // "add_tags" or "remove_tags"
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(
        database,
        action.to_string(),
        entity_type,
        entity_id,
        None,
    )
    .await
}

/// Helper to log goal operations on tasks
pub async fn log_goal_operation(
    database: Arc<Database>,
    action: &str, // "add_goal" or "remove_goal"
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(
        database,
        action.to_string(),
        entity_type,
        entity_id,
        None,
    )
    .await
}

/// Helper to log reorder action (for subtasks)
pub async fn log_reorder(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(database, "reorder".to_string(), entity_type, entity_id, None).await
}

/// Helper to log restore action (for trash)
pub async fn log_restore(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    log_activity(database, "restore".to_string(), entity_type, entity_id, None).await
}
