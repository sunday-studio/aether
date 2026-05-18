use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::{DateTime, Utc};
use libsql::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryInsight {
    pub id: String,
    pub entry_id: String,
    pub summary: String,
    pub possible_mood: Option<String>,
    pub emotions: Vec<String>,
    pub energy: Option<String>,
    pub themes: Vec<String>,
    pub people: Vec<String>,
    pub projects: Vec<String>,
    pub open_loops: Vec<String>,
    pub provider: String,
    pub model: Option<String>,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntrySuggestion {
    pub id: String,
    pub entry_id: String,
    pub insight_id: Option<String>,
    pub suggestion_type: String,
    pub value: String,
    pub edited_value: Option<String>,
    pub target_resource_type: Option<String>,
    pub target_resource_id: Option<String>,
    pub confidence: Option<f64>,
    pub provider: String,
    pub model: Option<String>,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyAiSummary {
    pub id: String,
    pub week_start: String,
    pub week_end: String,
    pub summary: String,
    pub themes: Vec<String>,
    pub completed_work: Vec<String>,
    pub open_loops: Vec<String>,
    pub next_focus: Vec<String>,
    pub source_entry_ids: Vec<String>,
    pub source_task_ids: Vec<String>,
    pub source_goal_ids: Vec<String>,
    pub provider: String,
    pub model: Option<String>,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryInsightBundle {
    pub insight: JournalEntryInsight,
    pub suggestions: Vec<JournalEntrySuggestion>,
}

#[derive(Debug, Clone)]
pub struct JournalEntryInsightInput {
    pub entry_id: String,
    pub summary: String,
    pub possible_mood: Option<String>,
    pub emotions: Vec<String>,
    pub energy: Option<String>,
    pub themes: Vec<String>,
    pub people: Vec<String>,
    pub projects: Vec<String>,
    pub open_loops: Vec<String>,
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JournalEntryInsightPatch {
    pub summary: Option<String>,
    pub possible_mood: Option<String>,
    pub emotions: Option<Vec<String>>,
    pub energy: Option<String>,
    pub themes: Option<Vec<String>>,
    pub people: Option<Vec<String>>,
    pub projects: Option<Vec<String>>,
    pub open_loops: Option<Vec<String>>,
    pub state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JournalEntrySuggestionInput {
    pub suggestion_type: String,
    pub value: String,
    pub target_resource_type: Option<String>,
    pub target_resource_id: Option<String>,
    pub confidence: Option<f64>,
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WeeklyAiSummaryInput {
    pub week_start: String,
    pub week_end: String,
    pub summary: String,
    pub themes: Vec<String>,
    pub completed_work: Vec<String>,
    pub open_loops: Vec<String>,
    pub next_focus: Vec<String>,
    pub source_entry_ids: Vec<String>,
    pub source_task_ids: Vec<String>,
    pub source_goal_ids: Vec<String>,
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WeeklyAiSummaryPatch {
    pub summary: Option<String>,
    pub themes: Option<Vec<String>>,
    pub completed_work: Option<Vec<String>>,
    pub open_loops: Option<Vec<String>>,
    pub next_focus: Option<Vec<String>>,
    pub state: Option<String>,
}

pub struct AiJournalEnrichmentRepository {
    database: Arc<Database>,
}

impl AiJournalEnrichmentRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub async fn upsert_entry_bundle(
        &self,
        input: JournalEntryInsightInput,
        suggestions: Vec<JournalEntrySuggestionInput>,
    ) -> Result<EntryInsightBundle> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();
        let insight_id = match self.get_entry_insight(&input.entry_id).await? {
            Some(existing) => existing.insight.id,
            None => generate_id("aiins"),
        };

        conn.execute(
            "INSERT INTO journal_entry_insights (
                id, entry_id, summary, possible_mood, emotions, energy, themes, people,
                projects, open_loops, provider, model, state, created_at, updated_at, deleted_at,
                _sync_id, _updated_at, _deleted, _extra
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'draft', ?13, ?14, NULL, ?1, ?15, 0, '{}')
            ON CONFLICT(entry_id) DO UPDATE SET
                summary = excluded.summary,
                possible_mood = excluded.possible_mood,
                emotions = excluded.emotions,
                energy = excluded.energy,
                themes = excluded.themes,
                people = excluded.people,
                projects = excluded.projects,
                open_loops = excluded.open_loops,
                provider = excluded.provider,
                model = excluded.model,
                state = 'draft',
                updated_at = excluded.updated_at,
                deleted_at = NULL,
                _sync_id = COALESCE(journal_entry_insights._sync_id, journal_entry_insights.id),
                _updated_at = excluded._updated_at,
                _deleted = 0,
                _extra = COALESCE(journal_entry_insights._extra, '{}')",
            libsql::params![
                insight_id.clone(),
                input.entry_id.clone(),
                input.summary,
                input.possible_mood,
                to_json_array(&input.emotions)?,
                input.energy,
                to_json_array(&input.themes)?,
                to_json_array(&input.people)?,
                to_json_array(&input.projects)?,
                to_json_array(&input.open_loops)?,
                input.provider,
                input.model,
                now_str,
                now.to_rfc3339(),
                now_ms,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        conn.execute(
            "UPDATE journal_entry_suggestions
             SET _deleted = 1, _updated_at = ?2, updated_at = ?3
             WHERE insight_id = ?1 AND state = 'pending' AND COALESCE(_deleted, 0) = 0",
            libsql::params![
                insight_id.clone(),
                Utc::now().timestamp_millis(),
                Utc::now().to_rfc3339()
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        for suggestion in suggestions {
            let id = generate_id("aisug");
            let suggestion_now = Utc::now();
            let created_at = suggestion_now.to_rfc3339();
            conn.execute(
                "INSERT INTO journal_entry_suggestions (
                    id, entry_id, insight_id, suggestion_type, value, edited_value,
                    target_resource_type, target_resource_id, confidence, provider, model,
                    state, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra
                )
                VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12, ?1, ?13, 0, '{}')",
                libsql::params![
                    id.clone(),
                    input.entry_id.clone(),
                    insight_id.clone(),
                    suggestion.suggestion_type,
                    suggestion.value,
                    suggestion.target_resource_type,
                    suggestion.target_resource_id,
                    suggestion.confidence,
                    suggestion.provider,
                    suggestion.model,
                    created_at,
                    suggestion_now.to_rfc3339(),
                    suggestion_now.timestamp_millis(),
                ],
            )
            .await
            .map_err(AppError::LibSQL)?;
        }

        self.get_entry_insight(&input.entry_id)
            .await?
            .ok_or_else(|| AppError::Internal("Entry insight was not created".to_string()))
    }

    pub async fn get_entry_insight(&self, entry_id: &str) -> Result<Option<EntryInsightBundle>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, entry_id, summary, possible_mood, emotions, energy, themes, people,
                    projects, open_loops, provider, model, state, created_at, updated_at, deleted_at
                 FROM journal_entry_insights
                 WHERE entry_id = ?1 AND deleted_at IS NULL AND COALESCE(_deleted, 0) = 0",
                libsql::params![entry_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let Some(row) = rows.next().await.map_err(AppError::LibSQL)? else {
            return Ok(None);
        };
        let insight = row_to_insight(row)?;
        let suggestions = self.list_entry_suggestions(entry_id).await?;
        Ok(Some(EntryInsightBundle {
            insight,
            suggestions,
        }))
    }

    pub async fn update_entry_insight(
        &self,
        insight_id: &str,
        patch: JournalEntryInsightPatch,
    ) -> Result<EntryInsightBundle> {
        if let Some(state) = patch.state.as_deref() {
            if !matches!(state, "draft" | "reviewed" | "dismissed") {
                return Err(AppError::BadRequest(format!(
                    "Unsupported insight state: {}",
                    state
                )));
            }
        }

        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now();
        conn.execute(
            "UPDATE journal_entry_insights
             SET summary = COALESCE(?1, summary),
                 possible_mood = COALESCE(?2, possible_mood),
                 emotions = COALESCE(?3, emotions),
                 energy = COALESCE(?4, energy),
                 themes = COALESCE(?5, themes),
                 people = COALESCE(?6, people),
                 projects = COALESCE(?7, projects),
                 open_loops = COALESCE(?8, open_loops),
                 state = COALESCE(?9, state),
                 updated_at = ?10,
                 _updated_at = ?11,
                 _sync_id = COALESCE(_sync_id, id)
             WHERE id = ?12 AND deleted_at IS NULL AND COALESCE(_deleted, 0) = 0",
            libsql::params![
                patch.summary,
                patch.possible_mood,
                optional_json_array(patch.emotions)?,
                patch.energy,
                optional_json_array(patch.themes)?,
                optional_json_array(patch.people)?,
                optional_json_array(patch.projects)?,
                optional_json_array(patch.open_loops)?,
                patch.state,
                now.to_rfc3339(),
                now.timestamp_millis(),
                insight_id,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        let entry_id = self
            .get_entry_id_for_insight(insight_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Insight {} not found", insight_id)))?;
        self.get_entry_insight(&entry_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Insight {} not found", insight_id)))
    }

    pub async fn list_entry_suggestions(
        &self,
        entry_id: &str,
    ) -> Result<Vec<JournalEntrySuggestion>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, entry_id, insight_id, suggestion_type, value, edited_value,
                    target_resource_type, target_resource_id, confidence, provider, model,
                    state, created_at, updated_at
                 FROM journal_entry_suggestions
                 WHERE entry_id = ?1 AND COALESCE(_deleted, 0) = 0
                 ORDER BY created_at ASC, id ASC",
                libsql::params![entry_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let mut suggestions = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            suggestions.push(row_to_suggestion(row)?);
        }
        Ok(suggestions)
    }

    pub async fn update_suggestion_state(
        &self,
        suggestion_id: &str,
        state: &str,
        edited_value: Option<String>,
    ) -> Result<JournalEntrySuggestion> {
        if !matches!(state, "pending" | "accepted" | "edited" | "dismissed") {
            return Err(AppError::BadRequest(format!(
                "Unsupported suggestion state: {}",
                state
            )));
        }

        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now();
        conn.execute(
            "UPDATE journal_entry_suggestions
             SET state = ?1, edited_value = ?2, updated_at = ?3,
                 _updated_at = ?4, _sync_id = COALESCE(_sync_id, id)
             WHERE id = ?5 AND COALESCE(_deleted, 0) = 0",
            libsql::params![
                state,
                edited_value,
                now.to_rfc3339(),
                now.timestamp_millis(),
                suggestion_id
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        self.get_suggestion(suggestion_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Suggestion {} not found", suggestion_id)))
    }

    pub async fn get_suggestion(
        &self,
        suggestion_id: &str,
    ) -> Result<Option<JournalEntrySuggestion>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, entry_id, insight_id, suggestion_type, value, edited_value,
                    target_resource_type, target_resource_id, confidence, provider, model,
                    state, created_at, updated_at
                 FROM journal_entry_suggestions
                 WHERE id = ?1 AND COALESCE(_deleted, 0) = 0",
                libsql::params![suggestion_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Ok(Some(row_to_suggestion(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn upsert_weekly_summary(
        &self,
        input: WeeklyAiSummaryInput,
    ) -> Result<WeeklyAiSummary> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        let existing_id = self
            .get_weekly_summary(&input.week_start, &input.week_end)
            .await?
            .map(|summary| summary.id)
            .unwrap_or_else(|| generate_id("aiweek"));

        conn.execute(
            "INSERT INTO weekly_ai_summaries (
                id, week_start, week_end, summary, themes, completed_work, open_loops,
                next_focus, source_entry_ids, source_task_ids, source_goal_ids,
                provider, model, state, created_at, updated_at, deleted_at,
                _sync_id, _updated_at, _deleted, _extra
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'draft', ?14, ?15, NULL, ?1, ?16, 0, '{}')
            ON CONFLICT(week_start, week_end) DO UPDATE SET
                summary = excluded.summary,
                themes = excluded.themes,
                completed_work = excluded.completed_work,
                open_loops = excluded.open_loops,
                next_focus = excluded.next_focus,
                source_entry_ids = excluded.source_entry_ids,
                source_task_ids = excluded.source_task_ids,
                source_goal_ids = excluded.source_goal_ids,
                provider = excluded.provider,
                model = excluded.model,
                state = 'draft',
                updated_at = excluded.updated_at,
                deleted_at = NULL,
                _sync_id = COALESCE(weekly_ai_summaries._sync_id, weekly_ai_summaries.id),
                _updated_at = excluded._updated_at,
                _deleted = 0,
                _extra = COALESCE(weekly_ai_summaries._extra, '{}')",
            libsql::params![
                existing_id,
                input.week_start.clone(),
                input.week_end.clone(),
                input.summary,
                to_json_array(&input.themes)?,
                to_json_array(&input.completed_work)?,
                to_json_array(&input.open_loops)?,
                to_json_array(&input.next_focus)?,
                to_json_array(&input.source_entry_ids)?,
                to_json_array(&input.source_task_ids)?,
                to_json_array(&input.source_goal_ids)?,
                input.provider,
                input.model,
                now.to_rfc3339(),
                now.to_rfc3339(),
                now_ms,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        self.get_weekly_summary(&input.week_start, &input.week_end)
            .await?
            .ok_or_else(|| AppError::Internal("Weekly AI summary was not created".to_string()))
    }

    pub async fn get_weekly_summary(
        &self,
        week_start: &str,
        week_end: &str,
    ) -> Result<Option<WeeklyAiSummary>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, week_start, week_end, summary, themes, completed_work,
                    open_loops, next_focus, source_entry_ids, source_task_ids,
                    source_goal_ids, provider, model, state, created_at, updated_at, deleted_at
                 FROM weekly_ai_summaries
                 WHERE week_start = ?1 AND week_end = ?2 AND deleted_at IS NULL AND COALESCE(_deleted, 0) = 0",
                libsql::params![week_start, week_end],
            )
            .await
            .map_err(AppError::LibSQL)?;

        if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Ok(Some(row_to_weekly_summary(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn update_weekly_summary(
        &self,
        summary_id: &str,
        patch: WeeklyAiSummaryPatch,
    ) -> Result<WeeklyAiSummary> {
        if let Some(state) = patch.state.as_deref() {
            if !matches!(state, "draft" | "reviewed" | "dismissed") {
                return Err(AppError::BadRequest(format!(
                    "Unsupported weekly summary state: {}",
                    state
                )));
            }
        }

        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now();
        conn.execute(
            "UPDATE weekly_ai_summaries
             SET summary = COALESCE(?1, summary),
                 themes = COALESCE(?2, themes),
                 completed_work = COALESCE(?3, completed_work),
                 open_loops = COALESCE(?4, open_loops),
                 next_focus = COALESCE(?5, next_focus),
                 state = COALESCE(?6, state),
                 updated_at = ?7,
                 _updated_at = ?8,
                 _sync_id = COALESCE(_sync_id, id)
             WHERE id = ?9 AND deleted_at IS NULL AND COALESCE(_deleted, 0) = 0",
            libsql::params![
                patch.summary,
                optional_json_array(patch.themes)?,
                optional_json_array(patch.completed_work)?,
                optional_json_array(patch.open_loops)?,
                optional_json_array(patch.next_focus)?,
                patch.state,
                now.to_rfc3339(),
                now.timestamp_millis(),
                summary_id,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        self.get_weekly_summary_by_id(summary_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Weekly summary {} not found", summary_id)))
    }

    async fn get_entry_id_for_insight(&self, insight_id: &str) -> Result<Option<String>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT entry_id FROM journal_entry_insights WHERE id = ?1 AND deleted_at IS NULL AND COALESCE(_deleted, 0) = 0",
                libsql::params![insight_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Ok(Some(row.get(0).map_err(AppError::LibSQL)?))
        } else {
            Ok(None)
        }
    }

    async fn get_weekly_summary_by_id(&self, summary_id: &str) -> Result<Option<WeeklyAiSummary>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, week_start, week_end, summary, themes, completed_work,
                    open_loops, next_focus, source_entry_ids, source_task_ids,
                    source_goal_ids, provider, model, state, created_at, updated_at, deleted_at
                 FROM weekly_ai_summaries
                 WHERE id = ?1 AND deleted_at IS NULL AND COALESCE(_deleted, 0) = 0",
                libsql::params![summary_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Ok(Some(row_to_weekly_summary(row)?))
        } else {
            Ok(None)
        }
    }
}

fn row_to_insight(row: libsql::Row) -> Result<JournalEntryInsight> {
    Ok(JournalEntryInsight {
        id: row.get(0).map_err(AppError::LibSQL)?,
        entry_id: row.get(1).map_err(AppError::LibSQL)?,
        summary: row.get(2).map_err(AppError::LibSQL)?,
        possible_mood: row.get(3).map_err(AppError::LibSQL)?,
        emotions: from_json_array(row.get::<String>(4).map_err(AppError::LibSQL)?)?,
        energy: row.get(5).map_err(AppError::LibSQL)?,
        themes: from_json_array(row.get::<String>(6).map_err(AppError::LibSQL)?)?,
        people: from_json_array(row.get::<String>(7).map_err(AppError::LibSQL)?)?,
        projects: from_json_array(row.get::<String>(8).map_err(AppError::LibSQL)?)?,
        open_loops: from_json_array(row.get::<String>(9).map_err(AppError::LibSQL)?)?,
        provider: row.get(10).map_err(AppError::LibSQL)?,
        model: row.get(11).map_err(AppError::LibSQL)?,
        state: row.get(12).map_err(AppError::LibSQL)?,
        created_at: parse_date(row.get::<String>(13).map_err(AppError::LibSQL)?)?,
        updated_at: parse_date(row.get::<String>(14).map_err(AppError::LibSQL)?)?,
        deleted_at: parse_optional_date(row.get(15).map_err(AppError::LibSQL)?)?,
    })
}

fn row_to_suggestion(row: libsql::Row) -> Result<JournalEntrySuggestion> {
    Ok(JournalEntrySuggestion {
        id: row.get(0).map_err(AppError::LibSQL)?,
        entry_id: row.get(1).map_err(AppError::LibSQL)?,
        insight_id: row.get(2).map_err(AppError::LibSQL)?,
        suggestion_type: row.get(3).map_err(AppError::LibSQL)?,
        value: row.get(4).map_err(AppError::LibSQL)?,
        edited_value: row.get(5).map_err(AppError::LibSQL)?,
        target_resource_type: row.get(6).map_err(AppError::LibSQL)?,
        target_resource_id: row.get(7).map_err(AppError::LibSQL)?,
        confidence: row.get(8).map_err(AppError::LibSQL)?,
        provider: row.get(9).map_err(AppError::LibSQL)?,
        model: row.get(10).map_err(AppError::LibSQL)?,
        state: row.get(11).map_err(AppError::LibSQL)?,
        created_at: parse_date(row.get::<String>(12).map_err(AppError::LibSQL)?)?,
        updated_at: parse_date(row.get::<String>(13).map_err(AppError::LibSQL)?)?,
    })
}

fn row_to_weekly_summary(row: libsql::Row) -> Result<WeeklyAiSummary> {
    Ok(WeeklyAiSummary {
        id: row.get(0).map_err(AppError::LibSQL)?,
        week_start: row.get(1).map_err(AppError::LibSQL)?,
        week_end: row.get(2).map_err(AppError::LibSQL)?,
        summary: row.get(3).map_err(AppError::LibSQL)?,
        themes: from_json_array(row.get::<String>(4).map_err(AppError::LibSQL)?)?,
        completed_work: from_json_array(row.get::<String>(5).map_err(AppError::LibSQL)?)?,
        open_loops: from_json_array(row.get::<String>(6).map_err(AppError::LibSQL)?)?,
        next_focus: from_json_array(row.get::<String>(7).map_err(AppError::LibSQL)?)?,
        source_entry_ids: from_json_array(row.get::<String>(8).map_err(AppError::LibSQL)?)?,
        source_task_ids: from_json_array(row.get::<String>(9).map_err(AppError::LibSQL)?)?,
        source_goal_ids: from_json_array(row.get::<String>(10).map_err(AppError::LibSQL)?)?,
        provider: row.get(11).map_err(AppError::LibSQL)?,
        model: row.get(12).map_err(AppError::LibSQL)?,
        state: row.get(13).map_err(AppError::LibSQL)?,
        created_at: parse_date(row.get::<String>(14).map_err(AppError::LibSQL)?)?,
        updated_at: parse_date(row.get::<String>(15).map_err(AppError::LibSQL)?)?,
        deleted_at: parse_optional_date(row.get(16).map_err(AppError::LibSQL)?)?,
    })
}

fn to_json_array(values: &[String]) -> Result<String> {
    serde_json::to_string(values).map_err(AppError::Serialization)
}

fn optional_json_array(values: Option<Vec<String>>) -> Result<Option<String>> {
    values
        .map(|items| serde_json::to_string(&items).map_err(AppError::Serialization))
        .transpose()
}

fn from_json_array(value: String) -> Result<Vec<String>> {
    serde_json::from_str(&value)
        .map_err(|e| AppError::Internal(format!("Invalid AI enrichment JSON array: {}", e)))
}

fn parse_date(value: String) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map_err(|e| AppError::Internal(format!("Invalid AI enrichment timestamp: {}", e)))
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_optional_date(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.map(parse_date).transpose()
}
