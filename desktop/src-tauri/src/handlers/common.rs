use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::models::{
    AudioTranscription, Bookmark, Canvas, Entry, Goal, GoalInstance, GoalInstanceWithTasks,
    ResourceLink, Tag, Task, TaskWithSubtasks,
};

/// Pagination response wrapper optimized for infinite scroll
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[aliases(
    PaginatedEntries = PaginationResponse<Entry>,
    PaginatedTags = PaginationResponse<Tag>,
    PaginatedTasks = PaginationResponse<Task>,
    PaginatedTasksWithSubtasks = PaginationResponse<TaskWithSubtasks>,
    PaginatedGoals = PaginationResponse<Goal>,
    PaginatedGoalInstances = PaginationResponse<GoalInstance>,
    PaginatedGoalInstancesWithTasks = PaginationResponse<GoalInstanceWithTasks>,
    PaginatedLinks = PaginationResponse<ResourceLink>,
    PaginatedBookmarks = PaginationResponse<Bookmark>,
    PaginatedCanvases = PaginationResponse<Canvas>,
    PaginatedTranscriptions = PaginationResponse<AudioTranscription>
)]
pub struct PaginationResponse<T> {
    pub items: Vec<T>,
    /// Next page cursor (null when no more pages)
    pub next_cursor: Option<String>,
    /// Convenience flag for infinite scroll
    pub has_more: bool,
}

impl<T> PaginationResponse<T> {
    /// Create a paginated response
    pub fn new(items: Vec<T>, next_cursor: Option<String>, has_more: bool) -> Self {
        Self {
            items,
            next_cursor,
            has_more,
        }
    }
    
    /// Create a response for all results (bypass mode)
    pub fn all(items: Vec<T>) -> Self {
        Self {
            items,
            next_cursor: None,
            has_more: false,
        }
    }
}

/// Cursor encoding/decoding utilities
pub mod cursor {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use crate::error::{AppError, Result};

    /// Encode a sort key (typically an ID) into a cursor
    pub fn encode(sort_key: &str) -> String {
        BASE64.encode(sort_key.as_bytes())
    }

    /// Decode a cursor back to a sort key
    pub fn decode(cursor: &str) -> Result<String> {
        BASE64
            .decode(cursor)
            .map_err(|e| AppError::BadRequest(format!("Invalid cursor format: {}", e)))
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| AppError::BadRequest(format!("Invalid cursor encoding: {}", e)))
            })
    }

    /// Encode composite sort keys (e.g., "name|id")
    pub fn encode_composite(keys: &[&str]) -> String {
        let combined = keys.join("|");
        encode(&combined)
    }

    /// Decode composite sort keys
    pub fn decode_composite(cursor: &str) -> Result<Vec<String>> {
        decode(cursor).map(|s| s.split('|').map(|k| k.to_string()).collect())
    }
}
