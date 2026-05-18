// Repository modules will be added in later milestones
// This module provides the repository pattern for database access

pub mod activity;
pub mod ai_journal_enrichment;
pub mod bookmark;
pub mod canvas;
pub mod entry;
pub mod goal;
pub mod link;
pub mod media;
pub mod search;
pub mod search_document;
pub mod search_embedding;
pub mod settings;
pub mod tag;
pub mod task;
pub mod transcription;

pub use activity::ActivityRepository;
pub use ai_journal_enrichment::{
    AiJournalEnrichmentRepository, EntryInsightBundle, JournalEntryInsight,
    JournalEntryInsightInput, JournalEntryInsightPatch, JournalEntrySuggestion,
    JournalEntrySuggestionInput, WeeklyAiSummary, WeeklyAiSummaryInput, WeeklyAiSummaryPatch,
};
pub use bookmark::BookmarkRepository;
pub use canvas::CanvasRepository;
pub use entry::EntryRepository;
pub use goal::GoalRepository;
pub use link::LinkRepository;
pub use media::MediaRepository;
pub use search::{ResourceType, SearchRepository, SearchResult};
pub use search_document::{SearchDocumentQuery, SearchDocumentRepository, SearchIndexStatus};
pub use search_embedding::{
    SearchEmbeddingInput, SearchEmbeddingModelStatus, SearchEmbeddingRepository,
    SearchEmbeddingStatus,
};
pub use settings::SettingsRepository;
pub use tag::TagRepository;
pub use task::TaskRepository;
pub use transcription::TranscriptionRepository;
