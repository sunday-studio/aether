use crate::db::models::Tag;
use crate::db::repositories::search_document::SearchDocumentResult;
use crate::db::repositories::{
    JournalEntryInsightInput, JournalEntrySuggestionInput, WeeklyAiSummaryInput,
};
use crate::error::{AppError, Result};
use crate::journal_ai::{EntryEnrichmentDraft, WeeklySummaryDraft};
use crate::utils::search_text::{normalize_search_text, truncate_preview};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

const OPENAI_PROVIDER: &str = "openai";
const RESPONSES_URL: &str = "https://api.openai.com/v1/responses";

pub struct OpenAiJournalAiProvider {
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct EntryOutput {
    summary: String,
    possible_mood: String,
    emotions: Vec<String>,
    energy: String,
    themes: Vec<String>,
    people: Vec<String>,
    projects: Vec<String>,
    open_loops: Vec<String>,
    tag_suggestions: Vec<OpenAiSuggestion>,
}

#[derive(Debug, Deserialize)]
struct OpenAiSuggestion {
    value: String,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct WeeklyOutput {
    summary: String,
    themes: Vec<String>,
    completed_work: Vec<String>,
    open_loops: Vec<String>,
    next_focus: Vec<String>,
}

impl OpenAiJournalAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }

    pub async fn build_entry_draft(
        &self,
        entry_id: &str,
        text: &str,
        tags: &[Tag],
        related: Vec<SearchDocumentResult>,
    ) -> Result<EntryEnrichmentDraft> {
        let normalized = normalize_search_text(text);
        let input = entry_prompt(&normalized, tags, &related);
        let output: EntryOutput = self
            .request_json(
                "aether_journal_entry_insight",
                entry_schema(),
                "Create editable journal insight suggestions. Use cautious non-diagnostic language. Never diagnose, prescribe, or claim certainty. Keep every field suitable for user review and editing.",
                &input,
            )
            .await?;

        let mut suggestions = output
            .tag_suggestions
            .into_iter()
            .filter(|suggestion| !suggestion.value.trim().is_empty())
            .map(|suggestion| JournalEntrySuggestionInput {
                suggestion_type: "tag".to_string(),
                value: suggestion.value,
                target_resource_type: None,
                target_resource_id: None,
                confidence: Some(suggestion.confidence.clamp(0.0, 1.0)),
                provider: OPENAI_PROVIDER.to_string(),
                model: Some(self.model.clone()),
            })
            .collect::<Vec<_>>();

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
                provider: OPENAI_PROVIDER.to_string(),
                model: Some(self.model.clone()),
            });
        }

        Ok(EntryEnrichmentDraft {
            insight: JournalEntryInsightInput {
                entry_id: entry_id.to_string(),
                summary: fallback_summary(output.summary, &normalized),
                possible_mood: non_empty(output.possible_mood),
                emotions: limit_strings(output.emotions, 8),
                energy: non_empty(output.energy),
                themes: limit_strings(output.themes, 8),
                people: limit_strings(output.people, 8),
                projects: limit_strings(output.projects, 8),
                open_loops: limit_strings(output.open_loops, 8),
                provider: OPENAI_PROVIDER.to_string(),
                model: Some(self.model.clone()),
            },
            suggestions,
        })
    }

    pub async fn build_weekly_summary_draft(
        &self,
        start_date: &str,
        end_date: &str,
        context: Vec<SearchDocumentResult>,
    ) -> Result<WeeklySummaryDraft> {
        let input = weekly_prompt(start_date, end_date, &context);
        let output: WeeklyOutput = self
            .request_json(
                "aether_weekly_journal_summary",
                weekly_schema(),
                "Create an editable weekly journal summary draft. Use cautious non-diagnostic language and only describe possible themes, work, and follow-ups from the supplied context.",
                &input,
            )
            .await?;

        let mut source_entry_ids = Vec::new();
        let mut source_task_ids = Vec::new();
        let mut source_goal_ids = Vec::new();
        for result in &context {
            match result.resource_type.as_str() {
                "entry" => push_unique(&mut source_entry_ids, &result.resource_id),
                "task" => push_unique(&mut source_task_ids, &result.resource_id),
                "goal" => push_unique(&mut source_goal_ids, &result.resource_id),
                _ => {}
            }
        }

        Ok(WeeklySummaryDraft {
            summary: WeeklyAiSummaryInput {
                week_start: start_date.to_string(),
                week_end: end_date.to_string(),
                summary: output.summary,
                themes: limit_strings(output.themes, 8),
                completed_work: limit_strings(output.completed_work, 8),
                open_loops: limit_strings(output.open_loops, 8),
                next_focus: limit_strings(output.next_focus, 5),
                source_entry_ids,
                source_task_ids,
                source_goal_ids,
                provider: OPENAI_PROVIDER.to_string(),
                model: Some(self.model.clone()),
            },
        })
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        schema_name: &str,
        schema: serde_json::Value,
        instructions: &str,
        input: &str,
    ) -> Result<T> {
        let body = json!({
            "model": self.model,
            "instructions": instructions,
            "input": input,
            "text": {
                "format": {
                    "type": "json_schema",
                    "name": schema_name,
                    "strict": true,
                    "schema": schema
                }
            }
        });

        let response = self
            .client
            .post(RESPONSES_URL)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let value: serde_json::Value = response.json().await?;
        if !status.is_success() {
            return Err(AppError::ModelError(format!(
                "OpenAI journal enrichment request failed {}: {}",
                status, value
            )));
        }

        let text = extract_output_text(&value).ok_or_else(|| {
            AppError::ModelError(
                "OpenAI journal enrichment response had no output text".to_string(),
            )
        })?;
        serde_json::from_str(&text).map_err(AppError::Serialization)
    }
}

fn entry_prompt(text: &str, tags: &[Tag], related: &[SearchDocumentResult]) -> String {
    let tag_names = tags
        .iter()
        .map(|tag| tag.name.as_str())
        .take(80)
        .collect::<Vec<_>>()
        .join(", ");
    let related_text = related
        .iter()
        .take(8)
        .map(|result| {
            format!(
                "- {} {}: {} — {}",
                result.resource_type, result.resource_id, result.title, result.preview
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Journal entry text:\n{}\n\nExisting tags:\n{}\n\nPossible related local resources:\n{}\n\nReturn only editable suggestions. Keep possible_mood and energy empty when unclear.",
        truncate_preview(text, 6000),
        tag_names,
        related_text
    )
}

fn weekly_prompt(start_date: &str, end_date: &str, context: &[SearchDocumentResult]) -> String {
    let context_text = context
        .iter()
        .take(80)
        .map(|result| {
            format!(
                "- {} {}: {} — {}",
                result.resource_type, result.resource_id, result.title, result.preview
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Week: {} through {}\n\nIndexed local context:\n{}\n\nReturn an editable weekly summary draft. Leave lists empty when evidence is weak.",
        start_date, end_date, context_text
    )
}

fn entry_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["summary", "possible_mood", "emotions", "energy", "themes", "people", "projects", "open_loops", "tag_suggestions"],
        "properties": {
            "summary": { "type": "string" },
            "possible_mood": { "type": "string" },
            "emotions": { "type": "array", "items": { "type": "string" } },
            "energy": { "type": "string" },
            "themes": { "type": "array", "items": { "type": "string" } },
            "people": { "type": "array", "items": { "type": "string" } },
            "projects": { "type": "array", "items": { "type": "string" } },
            "open_loops": { "type": "array", "items": { "type": "string" } },
            "tag_suggestions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["value", "confidence"],
                    "properties": {
                        "value": { "type": "string" },
                        "confidence": { "type": "number" }
                    }
                }
            }
        }
    })
}

fn weekly_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["summary", "themes", "completed_work", "open_loops", "next_focus"],
        "properties": {
            "summary": { "type": "string" },
            "themes": { "type": "array", "items": { "type": "string" } },
            "completed_work": { "type": "array", "items": { "type": "string" } },
            "open_loops": { "type": "array", "items": { "type": "string" } },
            "next_focus": { "type": "array", "items": { "type": "string" } }
        }
    })
}

fn extract_output_text(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(|item| item.as_str()) {
        return Some(text.to_string());
    }
    value
        .get("output")
        .and_then(|item| item.as_array())?
        .iter()
        .flat_map(|item| {
            item.get("content")
                .and_then(|content| content.as_array())
                .into_iter()
                .flatten()
        })
        .find_map(|content| content.get("text").and_then(|text| text.as_str()))
        .map(ToString::to_string)
}

fn fallback_summary(summary: String, text: &str) -> String {
    if summary.trim().is_empty() {
        if text.trim().is_empty() {
            "No journal text to summarize yet.".to_string()
        } else {
            truncate_preview(text, 240)
        }
    } else {
        summary
    }
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn limit_strings(values: Vec<String>, limit: usize) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .take(limit)
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}
