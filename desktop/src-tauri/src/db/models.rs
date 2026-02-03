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
    pub entity_type: String, // "entry" | "canvas" | "bookmark" | "task"
    pub entity_id: String,
    pub media_type: String, // "audio" | "image" | "video"
    pub file_path: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtasks: Option<Vec<SubTask>>,
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
}

/// Task with subtasks included
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskWithSubtasks {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dueDate")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "goalInstanceId")]
    pub goal_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "goalId")]
    pub goal_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
    pub subtasks: Vec<SubTask>,
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
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
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

/// Canvas model for storing JSON Canvas-compliant canvas data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub id: String,
    pub name: String,
    pub canvas_data: serde_json::Value, // JSON Canvas format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
}

/// Bookmark model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_json: Option<serde_json::Value>,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub _sync_id: Option<String>,
    #[serde(skip)]
    pub _updated_at: Option<i64>,
    #[serde(skip)]
    pub _deleted: bool,
    #[serde(skip)]
    pub _extra: Option<serde_json::Value>,
}

/// ResourceLink model for bidirectional linking between resources
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLink {
    pub id: String,
    pub source_type: String, // 'entry', 'task', 'goal', 'canvas', 'bookmark'
    pub source_id: String,
    pub target_type: String, // 'entry', 'task', 'goal', 'canvas', 'bookmark'
    pub target_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_text: Option<String>,
    pub created_at: DateTime<Utc>,
}
