# AI Journal Enrichment

This document records the planned AI journal enrichment direction. It is not shipped behavior yet.

## Implementation Status

- Added local tables for entry insights, entry suggestions, and weekly AI summaries in `010_ai_journal_enrichment.sql`.
- Added backend provider structure with the first local `rules` path for entry insight drafts and suggestion state updates.
- Added local weekly summary draft generation from indexed context.
- Added edit commands for entry insights and weekly summaries.
- Added accept flow for relation suggestions through normal resource links.
- Added journal settings keys and Settings UI for local rules, external context policy, and future OpenAI journal enrichment.
- Added review controls for entry insight drafts and weekly summary drafts.
- Added encrypted sync wiring for entry insights, entry suggestions, and weekly summaries.
- External provider generation logic is still planned work.

## Product Direction

Aether should use AI first as a quiet enrichment layer for journal work, not as a general chat surface. The first planned release should support daily entry insights, weekly summaries, and suggested relationships between journal entries, tasks, and goals.

Every AI-created value must remain editable and reviewable by the user. The journal body should not be rewritten automatically, and suggested tags, possible moods, possible themes, open loops, summaries, and relationship suggestions should all be accept, edit, or dismiss flows.

The UI should avoid diagnostic language. Use phrases such as "suggested tags", "possible mood", "possible themes", "reflection draft", and "AI summary draft". Do not present AI output as a diagnosis or as something Aether knows with certainty.

Grounded chat is deferred. It can be built later on top of accepted insights, weekly summaries, tags, links, and search results.

## Planned User Surfaces

### Daily Entry Insights

For each journal entry, Aether should be able to generate an editable insight draft containing:

- A short summary.
- Suggested ordinary tags.
- Separate structured fields for possible mood, emotions, energy, and themes.
- Mentioned people or projects when useful.
- Open loops or possible follow-ups.
- Suggested related entries, tasks, and goals.

Suggested ordinary tags become real tags only when the user accepts them. Mood, emotion, energy, and theme data should remain separate insight metadata rather than ordinary tags.

### Weekly Summaries

The weekly summary flow should gather journal entries, accepted or edited insights, completed and open tasks, goals touched, activity counts, accepted tags, and relevant relationship suggestions for a selected week.

The result should be an editable weekly rollup with:

- What happened.
- Important themes.
- Completed work.
- Open loops.
- Suggested focus for the next week.
- Links back to source entries, tasks, and goals.

### Relation Suggestions

Relation suggestions should use local search first. Aether should suggest possible links to existing entries, tasks, and goals, but accepted links should be created through the normal resource link system.

## AI Architecture

Add a backend AI module with provider abstraction rather than binding product behavior directly to one vendor.

Initial providers:

- `rules`: local-only fallback using existing tags, local search, activity data, and simple heuristics.
- `openai`: external provider using encrypted AI settings.
- `mock`: deterministic provider for tests and UI development.

Later-compatible providers:

- `groq`
- `ollama`
- `self_hosted`
- local model-backed generation

The same journal enrichment and weekly summary commands should work regardless of provider.

## Data Model

Add local tables for user-visible AI artifacts:

- `journal_entry_insights`: one editable insight record for an entry, including summary, possible mood, emotions, energy, themes, provider, model, state, and timestamps.
- `journal_entry_suggestions`: editable suggestions for `tag`, `theme`, `emotion`, `person`, `project`, `open_loop`, `related_entry`, `related_task`, and `related_goal`.
- `weekly_ai_summaries`: editable weekly rollups with source entry, task, and goal ids, summary sections, themes, open loops, provider, model, state, and timestamps.

Suggestions should track state:

- `pending`
- `accepted`
- `edited`
- `dismissed`

Temporary jobs, raw prompts, raw provider responses, embeddings, provider caches, and local model files should not be stored as synced user-visible artifacts.

## Command Surface

Planned Tauri commands:

- `enrich_journal_entry(entry_id, mode?)`
- `get_entry_insights(entry_id)`
- `update_entry_insight(insight_id, patch)`
- `update_ai_suggestion(suggestion_id, state, edited_value?)`
- `accept_ai_tag_suggestion(suggestion_id)`
- `accept_ai_relation_suggestion(suggestion_id)`
- `generate_weekly_ai_summary(start_date, end_date, mode?)`
- `get_weekly_ai_summary(start_date, end_date)`
- `update_weekly_ai_summary(summary_id, patch)`

Accepting a tag suggestion should create or reuse a normal tag and attach it to the journal entry. Accepting a relation suggestion should create the normal resource link.

## Settings

General AI generation settings should be separate from transcription settings.

Planned settings:

- `ai.enrichment.enabled`
- `ai.provider`: `rules` or `openai` for the first implementation
- `ai.external_context_policy`: `selected_context`, `full_period_context`, or `summaries_only`
- `ai.openai.api_key`
- `ai.openai.model`

External AI must be opt-in. Settings copy should explain what may be sent when an external provider is enabled.

## Sync And Privacy

The sync server should remain encrypted and AI-unaware. It should not inspect journal text, summaries, tags, embeddings, or AI metadata.

Sync these user-visible artifacts:

- Accepted or edited entry insights.
- Accepted, edited, or dismissed suggestion state.
- Weekly summaries.
- Tags and resource links created after the user accepts suggestions.

Do not sync by default:

- Embeddings.
- Raw prompts.
- Raw provider responses.
- Temporary jobs.
- Provider caches.
- Local model files.

Each device can rebuild local indexes and embeddings from synced source data.

## Test Plan

- Provider selection falls back to `rules` when external AI is disabled or unavailable.
- Provider JSON validation rejects malformed output without corrupting existing insights.
- Suggestion state transitions work for pending, accepted, edited, and dismissed states.
- Accepting a tag suggestion creates or reuses a tag and attaches it to the entry.
- Accepting a relation suggestion creates a normal resource link.
- Weekly summary context includes entries, accepted insights, tasks, goals, and activity counts for the requested date range.
- Sync pushes and pulls user-visible AI artifacts without syncing temporary jobs or raw provider data.
- The frontend renders pending suggestions and supports accept, dismiss, edit, and reviewed states.
- UI copy avoids diagnostic claims and presents AI output as suggested, possible, and editable.
