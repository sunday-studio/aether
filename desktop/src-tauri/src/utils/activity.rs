use crate::db::ActivityRepository;
use crate::error::Result;
use libsql::Database;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Log an activity to the database
pub async fn log_activity(
    database: Arc<Database>,
    action_type: String,
    entity_type: String,
    entity_id: String,
    metadata: Option<serde_json::Value>,
) -> Result<()> {
    info!(
        "Logging activity: action_type={}, entity_type={}, entity_id={}",
        action_type, entity_type, entity_id
    );
    
    let repo = ActivityRepository::new(database);
    match repo.create(action_type.clone(), entity_type.clone(), entity_id.clone(), metadata).await {
        Ok(activity) => {
            debug!(
                "Successfully logged activity: id={}, action_type={}, entity_type={}, entity_id={}",
                activity.id, activity.action_type, activity.entity_type, activity.entity_id
            );
            Ok(())
        }
        Err(e) => {
            error!(
                "Failed to log activity: action_type={}, entity_type={}, entity_id={}, error={}",
                action_type, entity_type, entity_id, e
            );
            Err(e)
        }
    }
}

/// Helper to log a create action
pub async fn log_create(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    debug!("log_create called: entity_type={}, entity_id={}", entity_type, entity_id);
    log_activity(database, "create".to_string(), entity_type, entity_id, None).await
}

/// Helper to log an update action
pub async fn log_update(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    debug!("log_update called: entity_type={}, entity_id={}", entity_type, entity_id);
    log_activity(database, "update".to_string(), entity_type, entity_id, None).await
}

/// Helper to log a delete action
pub async fn log_delete(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    debug!("log_delete called: entity_type={}, entity_id={}", entity_type, entity_id);
    log_activity(database, "delete".to_string(), entity_type, entity_id, None).await
}

/// Helper to log a complete action (for tasks/subtasks)
pub async fn log_complete(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    debug!("log_complete called: entity_type={}, entity_id={}", entity_type, entity_id);
    log_activity(database, "complete".to_string(), entity_type, entity_id, None).await
}

/// Helper to log tag operations
pub async fn log_tag_operation(
    database: Arc<Database>,
    action: &str, // "add_tags" or "remove_tags"
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    debug!("log_tag_operation called: action={}, entity_type={}, entity_id={}", action, entity_type, entity_id);
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
    debug!("log_goal_operation called: action={}, entity_type={}, entity_id={}", action, entity_type, entity_id);
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
    debug!("log_reorder called: entity_type={}, entity_id={}", entity_type, entity_id);
    log_activity(database, "reorder".to_string(), entity_type, entity_id, None).await
}

/// Helper to log restore action (for trash)
pub async fn log_restore(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
) -> Result<()> {
    debug!("log_restore called: entity_type={}, entity_id={}", entity_type, entity_id);
    log_activity(database, "restore".to_string(), entity_type, entity_id, None).await
}
