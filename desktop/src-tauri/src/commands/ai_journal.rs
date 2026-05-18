use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest};
use crate::db::repositories::{
    AiJournalEnrichmentRepository, EntryInsightBundle, JournalEntryInsightInput,
    JournalEntrySuggestion, JournalEntrySuggestionInput, SearchDocumentRepository, WeeklyAiSummary,
    WeeklyAiSummaryInput,
};
use crate::db::models::Tag;
use crate::db::{connection, DbState, EntryRepository, TagRepository};
use crate::error::{AppError, Result};
use crate::utils::search_text::{
    extract_text_from_lexical_document, first_search_line, normalize_search_text, truncate_preview,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use utoipa::ToSchema;

const RULES_PROVIDER: &str = "rules";

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
pub struct AcceptAiTagSuggestionRequest {
    pub suggestion_id: String,
    #[serde(default)]
    pub edited_value: Option<String>,
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
    let draft = build_rules_draft(&entry.id, &text, &tags, related);

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
        .update_suggestion_state(
            &request.suggestion_id,
            &request.state,
            request.edited_value,
        )
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
        .ok_or_else(|| AppError::NotFound(format!("Suggestion {} not found", request.suggestion_id)))?;
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
    let context = SearchDocumentRepository::new(db.clone())
        .list_context_by_date_range(&request.start_date, &request.end_date, Some(80))
        .await?;
    let input = build_weekly_rules_draft(&request.start_date, &request.end_date, context);

    AiJournalEnrichmentRepository::new(db)
        .upsert_weekly_summary(input)
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

struct RulesDraft {
    insight: JournalEntryInsightInput,
    suggestions: Vec<JournalEntrySuggestionInput>,
}

fn build_weekly_rules_draft(
    start_date: &str,
    end_date: &str,
    context: Vec<crate::db::repositories::search_document::SearchDocumentResult>,
) -> WeeklyAiSummaryInput {
    let mut source_entry_ids = Vec::new();
    let mut source_task_ids = Vec::new();
    let mut source_goal_ids = Vec::new();
    let mut completed_work = Vec::new();
    let mut open_loops = Vec::new();
    let mut themes = Vec::new();

    for result in &context {
        match result.resource_type.as_str() {
            "entry" => push_unique(&mut source_entry_ids, &result.resource_id),
            "task" => {
                push_unique(&mut source_task_ids, &result.resource_id);
                push_unique(&mut completed_work, &result.title);
            }
            "goal" => push_unique(&mut source_goal_ids, &result.resource_id),
            "tag" => push_unique(&mut themes, &result.title),
            _ => {}
        }

        let lower = result.preview.to_lowercase();
        if lower.contains("need to")
            || lower.contains("follow up")
            || lower.contains("remember to")
            || lower.contains("todo")
        {
            push_unique(&mut open_loops, &result.preview);
        }
    }

    let summary = if context.is_empty() {
        "No indexed journal context was found for this week yet.".to_string()
    } else {
        format!(
            "Draft summary for {} through {} based on {} indexed items.",
            start_date,
            end_date,
            context.len()
        )
    };
    let next_focus = open_loops.iter().take(3).cloned().collect();

    WeeklyAiSummaryInput {
        week_start: start_date.to_string(),
        week_end: end_date.to_string(),
        summary,
        themes: themes.into_iter().take(8).collect(),
        completed_work: completed_work.into_iter().take(8).collect(),
        open_loops: open_loops.into_iter().take(8).collect(),
        next_focus,
        source_entry_ids,
        source_task_ids,
        source_goal_ids,
        provider: RULES_PROVIDER.to_string(),
        model: None,
    }
}

fn build_rules_draft(
    entry_id: &str,
    text: &str,
    tags: &[crate::db::models::Tag],
    related: Vec<crate::db::repositories::search_document::SearchDocumentResult>,
) -> RulesDraft {
    let normalized = normalize_search_text(text);
    let lower = normalized.to_lowercase();
    let themes = detect_themes(&lower, tags);
    let emotions = detect_emotions(&lower);
    let possible_mood = detect_possible_mood(&lower, &emotions);
    let energy = detect_energy(&lower);
    let open_loops = detect_open_loops(&normalized);
    let summary = if normalized.is_empty() {
        "No journal text to summarize yet.".to_string()
    } else {
        truncate_preview(&first_search_line(&normalized), 240)
    };

    let mut suggestions = Vec::new();
    for theme in &themes {
        suggestions.push(tag_suggestion(theme, 0.72));
    }
    for emotion in &emotions {
        suggestions.push(simple_suggestion("emotion", emotion, 0.64));
    }
    for open_loop in &open_loops {
        suggestions.push(simple_suggestion("open_loop", open_loop, 0.58));
    }
    for result in related.into_iter().take(5) {
        suggestions.push(JournalEntrySuggestionInput {
            suggestion_type: match result.resource_type.as_str() {
                "task" => "related_task",
                "goal" => "related_goal",
                _ => "related_entry",
            }
            .to_string(),
            value: result.title,
            target_resource_type: Some(result.resource_type),
            target_resource_id: Some(result.resource_id),
            confidence: Some(result.score.min(1.0)),
            provider: RULES_PROVIDER.to_string(),
            model: None,
        });
    }

    RulesDraft {
        insight: JournalEntryInsightInput {
            entry_id: entry_id.to_string(),
            summary,
            possible_mood,
            emotions,
            energy,
            themes,
            people: Vec::new(),
            projects: Vec::new(),
            open_loops,
            provider: RULES_PROVIDER.to_string(),
            model: None,
        },
        suggestions,
    }
}

fn tag_suggestion(value: &str, confidence: f64) -> JournalEntrySuggestionInput {
    simple_suggestion("tag", value, confidence)
}

fn simple_suggestion(
    suggestion_type: &str,
    value: &str,
    confidence: f64,
) -> JournalEntrySuggestionInput {
    JournalEntrySuggestionInput {
        suggestion_type: suggestion_type.to_string(),
        value: value.to_string(),
        target_resource_type: None,
        target_resource_id: None,
        confidence: Some(confidence),
        provider: RULES_PROVIDER.to_string(),
        model: None,
    }
}

fn detect_themes(lower: &str, tags: &[crate::db::models::Tag]) -> Vec<String> {
    let mut themes = Vec::new();
    for tag in tags {
        let name = tag.name.trim().to_lowercase();
        if name.len() >= 3 && lower.contains(&name) {
            push_unique(&mut themes, tag.name.trim());
        }
    }

    for (needle, theme) in [
        ("work", "work"),
        ("project", "projects"),
        ("plan", "planning"),
        ("learn", "learning"),
        ("family", "family"),
        ("friend", "relationships"),
        ("money", "money"),
        ("travel", "travel"),
        ("write", "creative"),
    ] {
        if lower.contains(needle) {
            push_unique(&mut themes, theme);
        }
    }
    themes.truncate(8);
    themes
}

fn detect_emotions(lower: &str) -> Vec<String> {
    let mut emotions = Vec::new();
    for (needles, emotion) in [
        (&["happy", "good", "great", "proud"][..], "positive"),
        (&["excited", "energized", "motivated"][..], "excited"),
        (&["calm", "peaceful", "steady"][..], "calm"),
        (&["tired", "drained", "exhausted"][..], "tired"),
        (&["stuck", "blocked", "frustrated"][..], "frustrated"),
        (&["overwhelmed", "busy", "too much"][..], "overwhelmed"),
    ] {
        if needles.iter().any(|needle| lower.contains(needle)) {
            push_unique(&mut emotions, emotion);
        }
    }
    emotions
}

fn detect_possible_mood(lower: &str, emotions: &[String]) -> Option<String> {
    if emotions.iter().any(|value| value == "overwhelmed") {
        Some("possibly overwhelmed".to_string())
    } else if emotions.iter().any(|value| value == "frustrated") {
        Some("possibly frustrated".to_string())
    } else if lower.contains("grateful") {
        Some("possibly grateful".to_string())
    } else if emotions.iter().any(|value| value == "positive") {
        Some("possibly positive".to_string())
    } else {
        None
    }
}

fn detect_energy(lower: &str) -> Option<String> {
    if ["energized", "motivated", "focused"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("possibly high".to_string())
    } else if ["tired", "drained", "exhausted"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("possibly low".to_string())
    } else {
        None
    }
}

fn detect_open_loops(text: &str) -> Vec<String> {
    text.split(['.', '\n'])
        .map(str::trim)
        .filter(|line| {
            let lower = line.to_lowercase();
            line.contains('?')
                || lower.contains("need to")
                || lower.contains("follow up")
                || lower.contains("remember to")
                || lower.contains("todo")
        })
        .take(5)
        .map(|line| truncate_preview(line, 180))
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}
