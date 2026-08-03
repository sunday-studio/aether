//! Dependency-aware ordering for sync changes.
//!
//! Changes are stored and transported as an append-only stream, which does not
//! guarantee that an entity appears before rows with foreign keys to it.

use crate::sync::types::{ChangeEnvelope, ChangeOp};

/// Sort changes so referenced rows are created before dependent rows.
///
/// Rust's slice sort is stable, so changes in the same dependency tier retain
/// their server or outbox order.
pub fn sort_for_dependencies(changes: &mut [ChangeEnvelope]) {
    changes.sort_by_key(dependency_rank);
}

fn dependency_rank(change: &ChangeEnvelope) -> u8 {
    if change.op == ChangeOp::Delete {
        // Sync deletes are soft updates and do not need their parents. Apply
        // them after upserts so an older deferred upsert cannot be resurrected
        // by sorting ahead of its later delete.
        return 100;
    }

    match change.entity.as_str() {
        // No local foreign-key dependencies.
        "entries"
        | "tags"
        | "goals"
        | "canvases"
        | "bookmarks"
        | "media_items"
        | "activities"
        | "resource_links"
        | "weekly_ai_summaries" => 0,

        // Depend only on entities in the first tier.
        "goal_instances" | "audio_transcriptions" | "journal_entry_insights" => 10,

        // A task may reference a goal or a goal instance.
        "tasks" => 20,

        // Join rows and child records must be last.
        "entry_tags"
        | "goal_tags"
        | "goal_instance_tags"
        | "task_tags"
        | "bookmark_tags"
        | "subtasks"
        | "journal_entry_suggestions" => 30,

        // Unknown entities are applied after known base entities but preserve
        // their relative order with each other.
        _ => 25,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(entity: &str) -> ChangeEnvelope {
        ChangeEnvelope {
            entity: entity.into(),
            id: entity.into(),
            op: ChangeOp::Upsert,
            data: None,
            updated_at: 1,
            device_id: "device".into(),
            device_hostname: "host".into(),
        }
    }

    #[test]
    fn places_task_tag_parents_before_the_join_row() {
        let mut changes = vec![change("task_tags"), change("tasks"), change("tags")];

        sort_for_dependencies(&mut changes);

        assert_eq!(
            changes
                .iter()
                .map(|change| change.entity.as_str())
                .collect::<Vec<_>>(),
            ["tags", "tasks", "task_tags"]
        );
    }
}
