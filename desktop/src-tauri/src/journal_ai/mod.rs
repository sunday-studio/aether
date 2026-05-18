pub mod providers;

use crate::db::repositories::{
    JournalEntryInsightInput, JournalEntrySuggestionInput, WeeklyAiSummaryInput,
};

pub const RULES_PROVIDER: &str = "rules";

pub struct EntryEnrichmentDraft {
    pub insight: JournalEntryInsightInput,
    pub suggestions: Vec<JournalEntrySuggestionInput>,
}

pub struct WeeklySummaryDraft {
    pub summary: WeeklyAiSummaryInput,
}
