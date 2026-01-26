use serde::Deserialize;

/// Empty path parameters for commands with no path params
#[derive(Debug, Clone, Deserialize)]
pub struct EmptyPathParams {}

/// Empty query parameters for commands with no query params
#[derive(Debug, Clone, Deserialize)]
pub struct EmptyQueryParams {}

/// Empty request data for commands with no request body
#[derive(Debug, Clone, Deserialize)]
pub struct EmptyRequest {}

// ============================================================================
// Path Parameter Types
// ============================================================================

/// Path parameters for commands with a single ID (e.g., /:id)
#[derive(Debug, Clone, Deserialize)]
pub struct IdPathParams {
    pub id: String,
}

/// Path parameters for commands with media ID (e.g., /:mediaId)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaIdPathParams {
    pub media_id: String,
}

/// Path parameters for commands with task ID (e.g., /:taskId)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskIdPathParams {
    pub task_id: String,
}

/// Path parameters for commands with entry ID (e.g., /:entryId)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryIdPathParams {
    pub entry_id: String,
}

/// Path parameters for commands with goal ID (e.g., /:goalId)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalIdPathParams {
    pub goal_id: String,
}

/// Path parameters for commands with transcription ID (e.g., /:transcriptionId)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionIdPathParams {
    pub transcription_id: String,
}

/// Path parameters for commands with task ID and subtask ID (e.g., /:taskId/:subtaskId)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSubtaskPathParams {
    pub task_id: String,
    pub subtask_id: String,
}

/// Path parameters for commands with model size (e.g., /:modelSize)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSizePathParams {
    pub model_size: String,
}

/// Path parameters for commands with model name (e.g., /:modelName)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelNamePathParams {
    pub model_name: String,
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Query parameters for search commands
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQueryParams {
    pub q: String,
    #[serde(default)]
    pub types: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub mode: Option<String>,
}

/// Query parameters for bookmark listing
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BookmarkQueryParams {
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub tag_ids: Option<Vec<String>>,
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Query parameters for backlinks/outgoing links
#[derive(Debug, Clone, Deserialize)]
pub struct LinkQueryParams {
    pub resource_type: String,
    pub resource_id: String,
}

/// Query parameters for activity listing
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityQueryParams {
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

/// Query parameters for metadata extraction
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractMetadataQueryParams {
    pub url: String,
}

/// Query parameters for settings
#[derive(Debug, Clone, Deserialize)]
pub struct SettingQueryParams {
    pub key: String,
}

/// Query parameters for transcription start
#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionStartQueryParams {
    #[serde(default)]
    pub provider: Option<String>,
}
