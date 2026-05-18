use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest};
use crate::db::models::{ResourceLink, Tag};
use crate::db::repositories::{
    AiJournalEnrichmentRepository, EntryInsightBundle, JournalEntryInsightPatch,
    JournalEntrySuggestion, LinkRepository, SearchDocumentRepository, WeeklyAiSummary,
    WeeklyAiSummaryPatch,
};
use crate::db::{connection, DbState, EntryRepository, TagRepository};
use crate::error::{AppError, Result};
use crate::journal_ai::{
    providers::RulesJournalAiProvider, resolve_provider, JournalAiProviderKind,
};
use crate::utils::search_text::extract_text_from_lexical_document;
use serde::{Deserialize, Serialize};
use tauri::State;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnrichJournalEntryRequest {
    pub entry_id: String,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInsightQueryParams {
    pub entry_id: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyAiSummaryRequest {
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyAiSummaryQueryParams {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAiSuggestionRequest {
    pub suggestion_id: String,
    pub state: String,
    #[serde(default)]
    pub edited_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntryInsightRequest {
    pub insight_id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub possible_mood: Option<String>,
    #[serde(default)]
    pub emotions: Option<Vec<String>>,
    #[serde(default)]
    pub energy: Option<String>,
    #[serde(default)]
    pub themes: Option<Vec<String>>,
    #[serde(default)]
    pub people: Option<Vec<String>>,
    #[serde(default)]
    pub projects: Option<Vec<String>>,
    #[serde(default)]
    pub open_loops: Option<Vec<String>>,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWeeklyAiSummaryRequest {
    pub summary_id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub themes: Option<Vec<String>>,
    #[serde(default)]
    pub completed_work: Option<Vec<String>>,
    #[serde(default)]
    pub open_loops: Option<Vec<String>>,
    #[serde(default)]
    pub next_focus: Option<Vec<String>>,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptAiTagSuggestionRequest {
    pub suggestion_id: String,
    #[serde(default)]
    pub edited_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptAiRelationSuggestionRequest {
    pub suggestion_id: String,
    #[serde(default)]
    pub link_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestionResponse {
    pub suggestion: JournalEntrySuggestion,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptAiTagSuggestionResponse {
    pub suggestion: JournalEntrySuggestion,
    pub tag: Tag,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptAiRelationSuggestionResponse {
    pub suggestion: JournalEntrySuggestion,
    pub link: ResourceLink,
}

/// Generate an editable local AI insight draft for one journal entry.
#[utoipa::path(
    post,
    path = "/v1/ai/journal/entry/enrich",
    tag = "AI Journal",
    request_body = EnrichJournalEntryRequest,
    responses(
        (status = 200, description = "Editable entry insight draft", body = EntryInsightBundle),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entry not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn enrich_journal_entry(
    state: State<'_, DbState>,
    request_data: Option<EnrichJournalEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<EntryInsightBundle> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let provider = resolve_provider(db.clone(), request.mode.as_deref()).await?;
    let entry = EntryRepository::new(db.clone())
        .find_by_id(&request.entry_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", request.entry_id)))?;

    let text = extract_text_from_lexical_document(&entry.document)?;
    let (tags, _, _) = TagRepository::new(db.clone()).find_all(None, None).await?;
    let related = SearchDocumentRepository::new(db.clone())
        .find_related("entry", &entry.id, Some(5))
        .await
        .unwrap_or_default();
    let draft = match provider {
        JournalAiProviderKind::Rules => {
            RulesJournalAiProvider::build_entry_draft(&entry.id, &text, &tags, related)
        }
    };

    AiJournalEnrichmentRepository::new(db)
        .upsert_entry_bundle(draft.insight, draft.suggestions)
        .await
}

/// Get the current editable insight draft and suggestions for one journal entry.
#[utoipa::path(
    get,
    path = "/v1/ai/journal/entry/insights",
    tag = "AI Journal",
    params(
        ("entry_id" = String, Query, description = "Entry ID")
    ),
    responses(
        (status = 200, description = "Editable entry insight draft", body = EntryInsightBundle),
        (status = 404, description = "Insight not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_entry_insights(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<EntryInsightQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<EntryInsightBundle> {
    let _guard = connection::with_db_access(&*state).await;
    let params =
        query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".into()))?;
    AiJournalEnrichmentRepository::new(connection::get_database(&*state))
        .get_entry_insight(&params.entry_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No insight for entry {}", params.entry_id)))
}

/// Update editable fields for an entry insight draft.
#[utoipa::path(
    patch,
    path = "/v1/ai/journal/entry/insight",
    tag = "AI Journal",
    request_body = UpdateEntryInsightRequest,
    responses(
        (status = 200, description = "Updated entry insight draft", body = EntryInsightBundle),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Insight not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_entry_insight(
    state: State<'_, DbState>,
    request_data: Option<UpdateEntryInsightRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<EntryInsightBundle> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    AiJournalEnrichmentRepository::new(connection::get_database(&*state))
        .update_entry_insight(
            &request.insight_id,
            JournalEntryInsightPatch {
                summary: request.summary,
                possible_mood: request.possible_mood,
                emotions: request.emotions,
                energy: request.energy,
                themes: request.themes,
                people: request.people,
                projects: request.projects,
                open_loops: request.open_loops,
                state: request.state,
            },
        )
        .await
}

/// Update the review state for one AI suggestion.
#[utoipa::path(
    patch,
    path = "/v1/ai/journal/suggestion",
    tag = "AI Journal",
    request_body = UpdateAiSuggestionRequest,
    responses(
        (status = 200, description = "Updated suggestion", body = AiSuggestionResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Suggestion not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_ai_suggestion(
    state: State<'_, DbState>,
    request_data: Option<UpdateAiSuggestionRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<AiSuggestionResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let suggestion = AiJournalEnrichmentRepository::new(connection::get_database(&*state))
        .update_suggestion_state(&request.suggestion_id, &request.state, request.edited_value)
        .await?;
    Ok(AiSuggestionResponse { suggestion })
}

/// Accept a tag suggestion by creating or reusing a real tag and attaching it to the entry.
#[utoipa::path(
    post,
    path = "/v1/ai/journal/suggestion/accept-tag",
    tag = "AI Journal",
    request_body = AcceptAiTagSuggestionRequest,
    responses(
        (status = 200, description = "Accepted tag suggestion", body = AcceptAiTagSuggestionResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Suggestion not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn accept_ai_tag_suggestion(
    state: State<'_, DbState>,
    request_data: Option<AcceptAiTagSuggestionRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<AcceptAiTagSuggestionResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = AiJournalEnrichmentRepository::new(db.clone());
    let existing = repo
        .get_suggestion(&request.suggestion_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Suggestion {} not found", request.suggestion_id))
        })?;
    if existing.suggestion_type != "tag" {
        return Err(AppError::BadRequest(
            "Only tag suggestions can be accepted with this command".to_string(),
        ));
    }

    let tag_name = request
        .edited_value
        .clone()
        .or(existing.edited_value.clone())
        .unwrap_or_else(|| existing.value.clone())
        .trim()
        .to_string();
    if tag_name.is_empty() {
        return Err(AppError::BadRequest("Tag name cannot be empty".to_string()));
    }

    let tag_repo = TagRepository::new(db.clone());
    let (tags, _, _) = tag_repo.find_all(None, None).await?;
    let tag = match tags
        .into_iter()
        .find(|tag| tag.name.eq_ignore_ascii_case(&tag_name))
    {
        Some(tag) => tag,
        None => tag_repo.create(tag_name).await?,
    };

    EntryRepository::new(db.clone())
        .add_tags(&existing.entry_id, vec![tag.id.clone()])
        .await?;
    let suggestion = repo
        .update_suggestion_state(&existing.id, "accepted", request.edited_value)
        .await?;

    Ok(AcceptAiTagSuggestionResponse { suggestion, tag })
}

/// Accept a relation suggestion by creating a normal resource link.
#[utoipa::path(
    post,
    path = "/v1/ai/journal/suggestion/accept-relation",
    tag = "AI Journal",
    request_body = AcceptAiRelationSuggestionRequest,
    responses(
        (status = 200, description = "Accepted relation suggestion", body = AcceptAiRelationSuggestionResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Suggestion not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn accept_ai_relation_suggestion(
    state: State<'_, DbState>,
    request_data: Option<AcceptAiRelationSuggestionRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<AcceptAiRelationSuggestionResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = AiJournalEnrichmentRepository::new(db.clone());
    let existing = repo
        .get_suggestion(&request.suggestion_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Suggestion {} not found", request.suggestion_id))
        })?;
    if !matches!(
        existing.suggestion_type.as_str(),
        "related_entry" | "related_task" | "related_goal"
    ) {
        return Err(AppError::BadRequest(
            "Only relation suggestions can be accepted with this command".to_string(),
        ));
    }

    let target_type = existing.target_resource_type.clone().ok_or_else(|| {
        AppError::BadRequest("Relation suggestion has no target type".to_string())
    })?;
    let target_id = existing
        .target_resource_id
        .clone()
        .ok_or_else(|| AppError::BadRequest("Relation suggestion has no target id".to_string()))?;
    let link = LinkRepository::new(db.clone())
        .create(
            "entry".to_string(),
            existing.entry_id.clone(),
            target_type,
            target_id,
            request.link_text.or(Some(existing.value.clone())),
        )
        .await?;
    let suggestion = repo
        .update_suggestion_state(&existing.id, "accepted", existing.edited_value)
        .await?;

    Ok(AcceptAiRelationSuggestionResponse { suggestion, link })
}

/// Generate an editable local weekly summary draft.
#[utoipa::path(
    post,
    path = "/v1/ai/journal/weekly-summary",
    tag = "AI Journal",
    request_body = WeeklyAiSummaryRequest,
    responses(
        (status = 200, description = "Editable weekly summary draft", body = WeeklyAiSummary),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn generate_weekly_ai_summary(
    state: State<'_, DbState>,
    request_data: Option<WeeklyAiSummaryRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<WeeklyAiSummary> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.start_date.trim().is_empty() || request.end_date.trim().is_empty() {
        return Err(AppError::BadRequest(
            "startDate and endDate are required".to_string(),
        ));
    }

    let db = connection::get_database(&*state);
    let provider = resolve_provider(db.clone(), request.mode.as_deref()).await?;
    let context = SearchDocumentRepository::new(db.clone())
        .list_context_by_date_range(&request.start_date, &request.end_date, Some(80))
        .await?;
    let draft = match provider {
        JournalAiProviderKind::Rules => RulesJournalAiProvider::build_weekly_summary_draft(
            &request.start_date,
            &request.end_date,
            context,
        ),
    };

    AiJournalEnrichmentRepository::new(db)
        .upsert_weekly_summary(draft.summary)
        .await
}

/// Get an editable weekly summary draft.
#[utoipa::path(
    get,
    path = "/v1/ai/journal/weekly-summary",
    tag = "AI Journal",
    params(
        ("start_date" = String, Query, description = "Week start ISO 8601 value"),
        ("end_date" = String, Query, description = "Week end ISO 8601 value")
    ),
    responses(
        (status = 200, description = "Editable weekly summary draft", body = WeeklyAiSummary),
        (status = 404, description = "Weekly summary not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_weekly_ai_summary(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<WeeklyAiSummaryQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<WeeklyAiSummary> {
    let _guard = connection::with_db_access(&*state).await;
    let params =
        query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".into()))?;
    AiJournalEnrichmentRepository::new(connection::get_database(&*state))
        .get_weekly_summary(&params.start_date, &params.end_date)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "No weekly summary for {} through {}",
                params.start_date, params.end_date
            ))
        })
}

/// Update editable fields for a weekly summary draft.
#[utoipa::path(
    patch,
    path = "/v1/ai/journal/weekly-summary",
    tag = "AI Journal",
    request_body = UpdateWeeklyAiSummaryRequest,
    responses(
        (status = 200, description = "Updated weekly summary draft", body = WeeklyAiSummary),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Weekly summary not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_weekly_ai_summary(
    state: State<'_, DbState>,
    request_data: Option<UpdateWeeklyAiSummaryRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<WeeklyAiSummary> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    AiJournalEnrichmentRepository::new(connection::get_database(&*state))
        .update_weekly_summary(
            &request.summary_id,
            WeeklyAiSummaryPatch {
                summary: request.summary,
                themes: request.themes,
                completed_work: request.completed_work,
                open_loops: request.open_loops,
                next_focus: request.next_focus,
                state: request.state,
            },
        )
        .await
}
