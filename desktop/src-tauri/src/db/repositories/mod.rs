// Repository modules will be added in later milestones
// This module provides the repository pattern for database access

pub mod entry;
pub mod tag;
pub mod task;
pub mod goal;
pub mod activity;
pub mod media;
pub mod transcription;
pub mod settings;

pub use entry::EntryRepository;
pub use tag::TagRepository;
pub use task::TaskRepository;
pub use goal::GoalRepository;
pub use activity::ActivityRepository;
pub use media::MediaRepository;
pub use transcription::TranscriptionRepository;
pub use settings::SettingsRepository;
