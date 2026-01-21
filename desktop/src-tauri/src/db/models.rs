use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Schema migration tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMigration {
    pub id: i64,
    pub version: String,
    pub name: String,
    pub applied_at: DateTime<Utc>,
}

/// Settings model (key-value store)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

/// MediaItem model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub entry_id: String,
    pub media_type: String, // "audio" | "image" | "video"
    pub file_path: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AudioTranscription model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioTranscription {
    pub id: String,
    pub media_id: String,
    pub transcription_text: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f32>,
    pub status: String, // "pending" | "processing" | "complete" | "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Entry model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub document: String,
    pub created_at: DateTime<Utc>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Tag model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Task model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// SubTask model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub is_completed: bool,
    pub task_id: String,
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Goal model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_non_recurring: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_type: Option<String>, // bi-weekly | weekly | monthly | custom
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_interval: Option<i32>, // 1, 2, 25, etc
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_anchor: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_meta: Option<serde_json::Value>,
    pub timezone: String, // IANA timezone name, snapshot at creation
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// GoalInstance model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GoalInstance {
    pub id: String,
    pub goal_id: String,
    pub period_start: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_end: Option<DateTime<Utc>>, // nullable for non-recurring goals
    pub status: String, // active | completed | skipped
    pub created_at: DateTime<Utc>,
}

/// Activity model for tracking user actions and audit logging
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub action_type: String, // create, update, delete, complete, add_tags, remove_tags, add_goal, remove_goal, reorder, restore
    pub entity_type: String, // entry, task, subtask, goal, tag, goal_instance
    pub entity_id: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
