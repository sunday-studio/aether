use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Schema migration tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SchemaMigration {
    pub id: i64,
    pub version: String,
    pub name: String,
    pub applied_at: DateTime<Utc>,
}

/// Settings model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Settings {
    pub id: String,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Entry model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
pub struct GoalInstance {
    pub id: String,
    pub goal_id: String,
    pub period_start: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_end: Option<DateTime<Utc>>, // nullable for non-recurring goals
    pub status: String, // active | completed | skipped
    pub created_at: DateTime<Utc>,
}
