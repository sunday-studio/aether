pub mod providers;

use crate::db::repositories::{
    JournalEntryInsightInput, JournalEntrySuggestionInput, WeeklyAiSummaryInput,
};
use crate::error::{AppError, Result};
use crate::settings;
use libsql::Database;
use std::sync::Arc;

pub const RULES_PROVIDER: &str = "rules";
pub const ENRICHMENT_ENABLED_KEY: &str = "ai.enrichment.enabled";
pub const PROVIDER_KEY: &str = "ai.provider";
pub const EXTERNAL_CONTEXT_POLICY_KEY: &str = "ai.external_context_policy";
pub const OPENAI_API_KEY: &str = "ai.openai.api_key";
pub const OPENAI_MODEL_KEY: &str = "ai.openai.model";

pub struct EntryEnrichmentDraft {
    pub insight: JournalEntryInsightInput,
    pub suggestions: Vec<JournalEntrySuggestionInput>,
}

pub struct WeeklySummaryDraft {
    pub summary: WeeklyAiSummaryInput,
}

pub enum JournalAiProviderKind {
    Rules,
}

pub async fn resolve_provider(
    database: Arc<Database>,
    requested_mode: Option<&str>,
) -> Result<JournalAiProviderKind> {
    let enabled = settings::get_setting(database.clone(), ENRICHMENT_ENABLED_KEY).await?;
    if matches!(enabled.as_deref(), Some("false")) {
        return Err(AppError::BadRequest(
            "Journal AI enrichment is disabled in settings".to_string(),
        ));
    }

    let provider = match requested_mode {
        Some(mode) if !mode.trim().is_empty() => mode.trim().to_string(),
        _ => settings::get_setting(database, PROVIDER_KEY)
            .await?
            .unwrap_or_else(|| RULES_PROVIDER.to_string()),
    };

    match provider.as_str() {
        "" | RULES_PROVIDER | "local" => Ok(JournalAiProviderKind::Rules),
        "openai" => Err(AppError::BadRequest(
            "External journal AI is not implemented yet; use local rules for now".to_string(),
        )),
        value => Err(AppError::BadRequest(format!(
            "Unsupported journal AI provider '{}'",
            value
        ))),
    }
}
