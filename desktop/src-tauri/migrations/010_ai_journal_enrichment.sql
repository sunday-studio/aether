-- User-visible AI journal enrichment artifacts.
-- These tables store editable outputs only. Raw prompts, provider responses,
-- embeddings, jobs, caches, and model files are intentionally excluded.
-- Sync wiring is added separately so encrypted sync can handle these entities
-- explicitly instead of treating AI output as an unknown outbox entity.

CREATE TABLE IF NOT EXISTS journal_entry_insights (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    possible_mood TEXT,
    emotions TEXT NOT NULL DEFAULT '[]',
    energy TEXT,
    themes TEXT NOT NULL DEFAULT '[]',
    people TEXT NOT NULL DEFAULT '[]',
    projects TEXT NOT NULL DEFAULT '[]',
    open_loops TEXT NOT NULL DEFAULT '[]',
    provider TEXT NOT NULL DEFAULT 'rules',
    model TEXT,
    state TEXT NOT NULL DEFAULT 'draft'
        CHECK(state IN ('draft', 'reviewed', 'dismissed')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    UNIQUE(entry_id),
    FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_journal_entry_insights_entry
    ON journal_entry_insights(entry_id);

CREATE INDEX IF NOT EXISTS idx_journal_entry_insights_state
    ON journal_entry_insights(state);

CREATE INDEX IF NOT EXISTS idx_journal_entry_insights_updated
    ON journal_entry_insights(updated_at);

CREATE TABLE IF NOT EXISTS journal_entry_suggestions (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL,
    insight_id TEXT,
    suggestion_type TEXT NOT NULL
        CHECK(suggestion_type IN (
            'tag',
            'theme',
            'emotion',
            'person',
            'project',
            'open_loop',
            'related_entry',
            'related_task',
            'related_goal'
        )),
    value TEXT NOT NULL,
    edited_value TEXT,
    target_resource_type TEXT,
    target_resource_id TEXT,
    confidence REAL,
    provider TEXT NOT NULL DEFAULT 'rules',
    model TEXT,
    state TEXT NOT NULL DEFAULT 'pending'
        CHECK(state IN ('pending', 'accepted', 'edited', 'dismissed')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE,
    FOREIGN KEY(insight_id) REFERENCES journal_entry_insights(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_journal_entry_suggestions_entry
    ON journal_entry_suggestions(entry_id);

CREATE INDEX IF NOT EXISTS idx_journal_entry_suggestions_insight
    ON journal_entry_suggestions(insight_id);

CREATE INDEX IF NOT EXISTS idx_journal_entry_suggestions_state
    ON journal_entry_suggestions(state);

CREATE INDEX IF NOT EXISTS idx_journal_entry_suggestions_type
    ON journal_entry_suggestions(suggestion_type);

CREATE INDEX IF NOT EXISTS idx_journal_entry_suggestions_target
    ON journal_entry_suggestions(target_resource_type, target_resource_id);

CREATE TABLE IF NOT EXISTS weekly_ai_summaries (
    id TEXT PRIMARY KEY,
    week_start TEXT NOT NULL,
    week_end TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    themes TEXT NOT NULL DEFAULT '[]',
    completed_work TEXT NOT NULL DEFAULT '[]',
    open_loops TEXT NOT NULL DEFAULT '[]',
    next_focus TEXT NOT NULL DEFAULT '[]',
    source_entry_ids TEXT NOT NULL DEFAULT '[]',
    source_task_ids TEXT NOT NULL DEFAULT '[]',
    source_goal_ids TEXT NOT NULL DEFAULT '[]',
    provider TEXT NOT NULL DEFAULT 'rules',
    model TEXT,
    state TEXT NOT NULL DEFAULT 'draft'
        CHECK(state IN ('draft', 'reviewed', 'dismissed')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    UNIQUE(week_start, week_end)
);

CREATE INDEX IF NOT EXISTS idx_weekly_ai_summaries_week
    ON weekly_ai_summaries(week_start, week_end);

CREATE INDEX IF NOT EXISTS idx_weekly_ai_summaries_state
    ON weekly_ai_summaries(state);

CREATE INDEX IF NOT EXISTS idx_weekly_ai_summaries_updated
    ON weekly_ai_summaries(updated_at);
