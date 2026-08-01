// Repository modules will be added in later milestones
// This module provides the repository pattern for database access

pub mod activity;
pub mod entry;
pub mod goal;
pub mod media;
pub mod search_document;
pub mod search_embedding;
pub mod settings;
pub mod tag;
pub mod task;

pub use activity::ActivityRepository;
pub use entry::EntryRepository;
pub use goal::GoalRepository;
pub use media::MediaRepository;
pub use search_document::{SearchDocumentQuery, SearchDocumentRepository, SearchIndexStatus};
pub use search_embedding::{
    SearchEmbeddingInput, SearchEmbeddingModelStatus, SearchEmbeddingRepository,
    SearchEmbeddingStatus,
};
pub use settings::SettingsRepository;
pub use tag::TagRepository;
pub use task::TaskRepository;
