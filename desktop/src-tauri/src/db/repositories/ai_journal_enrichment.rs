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
        let insight_id = match self.get_entry_insight(&input.entry_id).await? {
            Some(existing) => existing.insight.id,
            None => generate_id("aiins"),
        };

        conn.execute(
            "INSERT INTO journal_entry_insights (
                id, entry_id, summary, possible_mood, emotions, energy, themes, people,
                projects, open_loops, provider, model, state, created_at, updated_at, deleted_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'draft', ?13, ?14, NULL)
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
                deleted_at = NULL",
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
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        conn.execute(
            "DELETE FROM journal_entry_suggestions WHERE insight_id = ?1 AND state = 'pending'",
            libsql::params![insight_id.clone()],
        )
        .await
        .map_err(AppError::LibSQL)?;

        for suggestion in suggestions {
            let id = generate_id("aisug");
            let created_at = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO journal_entry_suggestions (
                    id, entry_id, insight_id, suggestion_type, value, edited_value,
                    target_resource_type, target_resource_id, confidence, provider, model,
                    state, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12)",
                libsql::params![
                    id,
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
                    Utc::now().to_rfc3339(),
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
                 WHERE entry_id = ?1 AND deleted_at IS NULL",
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
                 WHERE entry_id = ?1
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
        conn.execute(
            "UPDATE journal_entry_suggestions
             SET state = ?1, edited_value = ?2, updated_at = ?3
             WHERE id = ?4",
            libsql::params![state, edited_value, Utc::now().to_rfc3339(), suggestion_id],
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
                 WHERE id = ?1",
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

fn to_json_array(values: &[String]) -> Result<String> {
    serde_json::to_string(values).map_err(AppError::Serialization)
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
