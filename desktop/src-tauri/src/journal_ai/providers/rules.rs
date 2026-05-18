use crate::db::models::Tag;
use crate::db::repositories::search_document::SearchDocumentResult;
use crate::db::repositories::{
    JournalEntryInsightInput, JournalEntrySuggestionInput, WeeklyAiSummaryInput,
};
use crate::journal_ai::{EntryEnrichmentDraft, WeeklySummaryDraft, RULES_PROVIDER};
use crate::utils::search_text::{first_search_line, normalize_search_text, truncate_preview};

pub struct RulesJournalAiProvider;

impl RulesJournalAiProvider {
    pub fn build_entry_draft(
        entry_id: &str,
        text: &str,
        tags: &[Tag],
        related: Vec<SearchDocumentResult>,
    ) -> EntryEnrichmentDraft {
        let normalized = normalize_search_text(text);
        let lower = normalized.to_lowercase();
        let themes = detect_themes(&lower, tags);
        let emotions = detect_emotions(&lower);
        let possible_mood = detect_possible_mood(&emotions, &lower);
        let energy = detect_energy(&lower);
        let open_loops = detect_open_loops(&normalized);
        let summary = if normalized.is_empty() {
            "No journal text to summarize yet.".to_string()
        } else {
            truncate_preview(&first_search_line(&normalized), 240)
        };

        let mut suggestions = Vec::new();
        for theme in &themes {
            suggestions.push(tag_suggestion(theme, 0.72));
        }
        for emotion in &emotions {
            suggestions.push(simple_suggestion("emotion", emotion, 0.64));
        }
        for open_loop in &open_loops {
            suggestions.push(simple_suggestion("open_loop", open_loop, 0.58));
        }
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
                provider: RULES_PROVIDER.to_string(),
                model: None,
            });
        }

        EntryEnrichmentDraft {
            insight: JournalEntryInsightInput {
                entry_id: entry_id.to_string(),
                summary,
                possible_mood,
                emotions,
                energy,
                themes,
                people: Vec::new(),
                projects: Vec::new(),
                open_loops,
                provider: RULES_PROVIDER.to_string(),
                model: None,
            },
            suggestions,
        }
    }

    pub fn build_weekly_summary_draft(
        start_date: &str,
        end_date: &str,
        context: Vec<SearchDocumentResult>,
    ) -> WeeklySummaryDraft {
        let mut source_entry_ids = Vec::new();
        let mut source_task_ids = Vec::new();
        let mut source_goal_ids = Vec::new();
        let mut completed_work = Vec::new();
        let mut open_loops = Vec::new();
        let mut themes = Vec::new();

        for result in &context {
            match result.resource_type.as_str() {
                "entry" => push_unique(&mut source_entry_ids, &result.resource_id),
                "task" => {
                    push_unique(&mut source_task_ids, &result.resource_id);
                    push_unique(&mut completed_work, &result.title);
                }
                "goal" => push_unique(&mut source_goal_ids, &result.resource_id),
                "tag" => push_unique(&mut themes, &result.title),
                _ => {}
            }

            let lower = result.preview.to_lowercase();
            if lower.contains("need to")
                || lower.contains("follow up")
                || lower.contains("remember to")
                || lower.contains("todo")
            {
                push_unique(&mut open_loops, &result.preview);
            }
        }

        let summary = if context.is_empty() {
            "No indexed journal context was found for this week yet.".to_string()
        } else {
            format!(
                "Draft summary for {} through {} based on {} indexed items.",
                start_date,
                end_date,
                context.len()
            )
        };
        let next_focus = open_loops.iter().take(3).cloned().collect();

        WeeklySummaryDraft {
            summary: WeeklyAiSummaryInput {
                week_start: start_date.to_string(),
                week_end: end_date.to_string(),
                summary,
                themes: themes.into_iter().take(8).collect(),
                completed_work: completed_work.into_iter().take(8).collect(),
                open_loops: open_loops.into_iter().take(8).collect(),
                next_focus,
                source_entry_ids,
                source_task_ids,
                source_goal_ids,
                provider: RULES_PROVIDER.to_string(),
                model: None,
            },
        }
    }
}

fn tag_suggestion(value: &str, confidence: f64) -> JournalEntrySuggestionInput {
    simple_suggestion("tag", value, confidence)
}

fn simple_suggestion(
    suggestion_type: &str,
    value: &str,
    confidence: f64,
) -> JournalEntrySuggestionInput {
    JournalEntrySuggestionInput {
        suggestion_type: suggestion_type.to_string(),
        value: value.to_string(),
        target_resource_type: None,
        target_resource_id: None,
        confidence: Some(confidence),
        provider: RULES_PROVIDER.to_string(),
        model: None,
    }
}

fn detect_themes(lower: &str, tags: &[Tag]) -> Vec<String> {
    let mut themes = Vec::new();
    for tag in tags {
        let name = tag.name.trim().to_lowercase();
        if name.len() >= 3 && lower.contains(&name) {
            push_unique(&mut themes, tag.name.trim());
        }
    }

    for (needle, theme) in [
        ("work", "work"),
        ("project", "projects"),
        ("plan", "planning"),
        ("learn", "learning"),
        ("family", "family"),
        ("friend", "relationships"),
        ("money", "money"),
        ("travel", "travel"),
        ("write", "creative"),
    ] {
        if lower.contains(needle) {
            push_unique(&mut themes, theme);
        }
    }
    themes.truncate(8);
    themes
}

fn detect_emotions(lower: &str) -> Vec<String> {
    let mut emotions = Vec::new();
    for (needles, emotion) in [
        (&["happy", "good", "great", "proud"][..], "positive"),
        (&["excited", "energized", "motivated"][..], "excited"),
        (&["calm", "peaceful", "steady"][..], "calm"),
        (&["tired", "drained", "exhausted"][..], "tired"),
        (&["stuck", "blocked", "frustrated"][..], "frustrated"),
        (&["overwhelmed", "busy", "too much"][..], "overwhelmed"),
    ] {
        if needles.iter().any(|needle| lower.contains(needle)) {
            push_unique(&mut emotions, emotion);
        }
    }
    emotions
}

fn detect_possible_mood(emotions: &[String], lower: &str) -> Option<String> {
    if emotions.iter().any(|value| value == "overwhelmed") {
        Some("possibly overwhelmed".to_string())
    } else if emotions.iter().any(|value| value == "frustrated") {
        Some("possibly frustrated".to_string())
    } else if lower.contains("grateful") {
        Some("possibly grateful".to_string())
    } else if emotions.iter().any(|value| value == "positive") {
        Some("possibly positive".to_string())
    } else {
        None
    }
}

fn detect_energy(lower: &str) -> Option<String> {
    if ["energized", "motivated", "focused"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("possibly high".to_string())
    } else if ["tired", "drained", "exhausted"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("possibly low".to_string())
    } else {
        None
    }
}

fn detect_open_loops(text: &str) -> Vec<String> {
    text.split(['.', '\n'])
        .map(str::trim)
        .filter(|line| {
            let lower = line.to_lowercase();
            line.contains('?')
                || lower.contains("need to")
                || lower.contains("follow up")
                || lower.contains("remember to")
                || lower.contains("todo")
        })
        .take(5)
        .map(|line| truncate_preview(line, 180))
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}
