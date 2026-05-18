use utoipa::OpenApi;

use crate::commands::activity as activity_commands;
use crate::commands::ai_journal as ai_journal_commands;
use crate::commands::bookmark as bookmark_commands;
use crate::commands::canvas as canvas_commands;
use crate::commands::common::{
    PaginatedBookmarks, PaginatedCanvases, PaginatedEntries, PaginatedGoalInstances,
    PaginatedGoals, PaginatedLinks, PaginatedTags, PaginatedTasks, PaginatedTasksWithSubtasks,
    PaginatedTranscriptions,
};
use crate::commands::embeddings as embedding_commands;
use crate::commands::entry as entry_commands;
use crate::commands::goal as goal_commands;
use crate::commands::link as link_commands;
use crate::commands::search as search_commands;
use crate::commands::settings as settings_commands;
use crate::commands::tag as tag_commands;
use crate::commands::task as task_commands;
use crate::commands::trash as trash_commands;
use crate::commands::{
    audio as audio_commands, sync as sync_commands, transcription as transcription_commands,
};
use crate::db::models::{
    Activity, AudioTranscription, Bookmark, Canvas, Entry, Goal, GoalInstance, MediaItem,
    ResourceLink, SubTask, Tag, Task, TaskWithSubtasks,
};
use crate::utils::metadata::extractor::ExtractedMetadata;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Tag endpoints
        tag_commands::get_all_tags,
        tag_commands::create_tag,
        tag_commands::bulk_create_tags,
        // Entry endpoints
        entry_commands::get_entries,
        entry_commands::get_entry_by_id,
        entry_commands::create_entry,
        entry_commands::bulk_create_entries,
        entry_commands::update_entry,
        entry_commands::delete_entry,
        entry_commands::add_tags_to_entry,
        entry_commands::remove_tags_from_entry,
        // Task endpoints
        task_commands::create_task,
        task_commands::get_inbox_tasks,
        task_commands::get_overdue_tasks,
        task_commands::get_task_by_id,
        task_commands::update_task,
        task_commands::delete_task,
        task_commands::get_subtasks,
        task_commands::create_subtask,
        task_commands::update_subtask,
        task_commands::delete_subtask,
        task_commands::reorder_subtasks,
        task_commands::add_tags_to_task,
        task_commands::remove_tags_from_task,
        task_commands::add_goal_to_task,
        task_commands::remove_goal_from_task,
        // Goal endpoints
        goal_commands::get_goals,
        goal_commands::get_goal_by_id,
        goal_commands::create_goal,
        goal_commands::update_goal,
        goal_commands::delete_goal,
        goal_commands::get_goal_instances,
        goal_commands::get_current_goal_instance,
        goal_commands::add_tags_to_goal,
        goal_commands::remove_tags_from_goal,
        // Trash endpoints
        trash_commands::get_trashed_tasks,
        trash_commands::restore_task,
        // Activity endpoints
        activity_commands::get_activities,
        // Search endpoints
        search_commands::search_resources,
        search_commands::reindex_search,
        search_commands::reindex_search_resource,
        search_commands::get_search_index_status,
        search_commands::find_related_resources,
        search_commands::retrieve_context,
        search_commands::retrieve_week_context,
        embedding_commands::index_search_embeddings,
        embedding_commands::index_search_resource_embeddings,
        embedding_commands::get_search_embedding_status,
        // AI journal endpoints
        ai_journal_commands::enrich_journal_entry,
        ai_journal_commands::get_entry_insights,
        ai_journal_commands::update_entry_insight,
        ai_journal_commands::update_ai_suggestion,
        ai_journal_commands::accept_ai_tag_suggestion,
        ai_journal_commands::accept_ai_relation_suggestion,
        ai_journal_commands::generate_weekly_ai_summary,
        ai_journal_commands::get_weekly_ai_summary,
        ai_journal_commands::update_weekly_ai_summary,
        // Link endpoints
        link_commands::create_link,
        link_commands::get_backlinks,
        link_commands::get_outgoing_links,
        link_commands::delete_link,
        link_commands::search_linkable_resources,
        link_commands::get_all_links_for_graph,
        link_commands::sync_links_from_content,
        // Settings endpoints
        settings_commands::get_setting,
        settings_commands::get_all_settings,
        settings_commands::set_setting,
        // Audio endpoints
        audio_commands::save_audio_recording,
        audio_commands::get_audio_data,
        audio_commands::delete_audio_recording,
        audio_commands::get_media_items_for_entry,
        audio_commands::get_audio_metadata,
        // Transcription endpoints
        transcription_commands::start_transcription,
        transcription_commands::get_transcriptions,
        transcription_commands::get_transcription_by_id,
        transcription_commands::set_active_transcription,
        transcription_commands::list_providers,
        transcription_commands::validate_provider,
        transcription_commands::list_available_models,
        transcription_commands::download_model,
        transcription_commands::verify_model,
        transcription_commands::delete_model,
        // Sync endpoints
        sync_commands::configure_sync,
        sync_commands::sync_now,
        sync_commands::get_sync_status,
        sync_commands::disconnect_sync,
        sync_commands::reconnect_sync,
        sync_commands::ensure_media_blob,
        // Bookmark endpoints
        bookmark_commands::get_bookmarks,
        bookmark_commands::get_bookmark_by_id,
        bookmark_commands::create_bookmark,
        bookmark_commands::update_bookmark,
        bookmark_commands::delete_bookmark,
        bookmark_commands::add_tags_to_bookmark,
        bookmark_commands::remove_tags_from_bookmark,
        bookmark_commands::extract_metadata,
        // Canvas endpoints
        canvas_commands::get_canvases,
        canvas_commands::get_canvas_by_id,
        canvas_commands::create_canvas,
        canvas_commands::update_canvas,
        canvas_commands::delete_canvas,
    ),
    components(schemas(
        // Base models
        Tag,
        Entry,
        Task,
        SubTask,
        Goal,
        GoalInstance,
        Activity,
        ResourceLink,
        Canvas,
        Bookmark,
        MediaItem,
        AudioTranscription,
        ExtractedMetadata,
        // Pagination response types (concrete aliases for proper OpenAPI generation)
        PaginatedEntries,
        PaginatedTags,
        PaginatedTasks,
        PaginatedTasksWithSubtasks,
        PaginatedGoals,
        PaginatedGoalInstances,
        PaginatedLinks,
        PaginatedCanvases,
        PaginatedBookmarks,
        PaginatedTranscriptions,
        // Composite types
        TaskWithSubtasks,
        // Request/Response schemas
        tag_commands::CreateTagRequest,
        entry_commands::CreateEntryRequest,
        entry_commands::UpdateEntryRequest,
        task_commands::CreateTaskRequest,
        task_commands::UpdateTaskRequest,
        task_commands::CreateSubTaskRequest,
        task_commands::UpdateSubTaskRequest,
        task_commands::ReorderSubTasksRequest,
        task_commands::AddGoalToTaskRequest,
        goal_commands::CreateGoalRequest,
        goal_commands::UpdateGoalRequest,
        search_commands::SearchRequest,
        search_commands::SearchResponse,
        search_commands::SearchContextResponse,
        search_commands::ReindexResourceRequest,
        crate::db::repositories::SearchIndexStatus,
        embedding_commands::IndexSearchEmbeddingsRequest,
        embedding_commands::IndexSearchResourceEmbeddingsRequest,
        crate::db::repositories::SearchEmbeddingStatus,
        crate::db::repositories::EntryInsightBundle,
        crate::db::repositories::JournalEntryInsight,
        crate::db::repositories::JournalEntrySuggestion,
        crate::db::repositories::WeeklyAiSummary,
        ai_journal_commands::EnrichJournalEntryRequest,
        ai_journal_commands::UpdateEntryInsightRequest,
        ai_journal_commands::UpdateAiSuggestionRequest,
        ai_journal_commands::AiSuggestionResponse,
        ai_journal_commands::AcceptAiTagSuggestionRequest,
        ai_journal_commands::AcceptAiTagSuggestionResponse,
        ai_journal_commands::AcceptAiRelationSuggestionRequest,
        ai_journal_commands::AcceptAiRelationSuggestionResponse,
        ai_journal_commands::WeeklyAiSummaryRequest,
        ai_journal_commands::UpdateWeeklyAiSummaryRequest,
        link_commands::CreateLinkRequest,
        link_commands::LinkableResource,
        link_commands::BacklinkResponse,
        link_commands::SyncLinksRequest,
        settings_commands::SettingResponse,
        settings_commands::AllSettingsResponse,
        settings_commands::SetSettingRequest,
        transcription_commands::ProviderInfo,
        transcription_commands::ModelInfo,
        transcription_commands::SetActiveTranscriptionRequest,
        transcription_commands::ValidateProviderRequest,
        crate::sync::SyncStatus,
        sync_commands::ConfigureSyncRequest,
        sync_commands::ReconnectSyncRequest,
        bookmark_commands::CreateBookmarkRequest,
        bookmark_commands::UpdateBookmarkRequest,
        bookmark_commands::AddTagsToBookmarkRequest,
        bookmark_commands::RemoveTagsFromBookmarkRequest,
        canvas_commands::CreateCanvasRequest,
        canvas_commands::UpdateCanvasRequest,
        audio_commands::SaveAudioRecordingRequest,
    )),
    tags(
        (name = "Tags", description = "Tag management endpoints"),
        (name = "Entries", description = "Entry management endpoints"),
        (name = "Tasks", description = "Task management endpoints"),
        (name = "Goals", description = "Goal management endpoints"),
        (name = "GoalInstances", description = "Goal instance management endpoints"),
        (name = "Trash", description = "Trash management endpoints"),
        (name = "Activities", description = "Activity tracking endpoints"),
        (name = "Search", description = "Search endpoints"),
        (name = "AI Journal", description = "AI journal enrichment endpoints"),
        (name = "Links", description = "Resource linking endpoints"),
        (name = "Settings", description = "Settings management endpoints"),
        (name = "Audio", description = "Audio recording and playback endpoints"),
        (name = "Transcription", description = "Audio transcription endpoints"),
        (name = "Sync", description = "Sync management endpoints"),
        (name = "Bookmarks", description = "Bookmark management endpoints"),
        (name = "Canvases", description = "Canvas management endpoints"),
    ),
)]
pub struct ApiDoc;

/// Get the OpenAPI spec as JSON string
/// This is used for build-time spec generation
pub fn get_openapi_json() -> String {
    let openapi = ApiDoc::openapi();
    serde_json::to_string_pretty(&openapi).unwrap_or_else(|e| {
        panic!("Failed to serialize OpenAPI spec: {}", e);
    })
}

// pub fn swagger_ui() -> SwaggerUi {
//     SwaggerUi::new("/swagger/{*path}").url("/api-doc/openapi.json", ApiDoc::openapi())
// }
