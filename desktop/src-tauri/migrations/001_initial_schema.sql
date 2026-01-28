-- Initial schema migration
-- Creates all core application tables with complete structure from the start

-- Settings table (key-value store from the start)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_tags_deleted_at ON tags(deleted_at);

-- Entries table
CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY,
    document TEXT NOT NULL,
    created_at TEXT NOT NULL,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_entries_deleted_at ON entries(deleted_at);
CREATE INDEX IF NOT EXISTS idx_entries_created_at ON entries(created_at);

-- Entry-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS entry_tags (
    entry_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (entry_id, tag_id),
    FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Goals table
CREATE TABLE IF NOT EXISTS goals (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_non_recurring INTEGER NOT NULL DEFAULT 0,
    recurrence_type TEXT,
    recurrence_interval INTEGER,
    recurrence_anchor TEXT,
    recurrence_meta TEXT,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_goals_deleted_at ON goals(deleted_at);

-- Goal-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS goal_tags (
    goal_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (goal_id, tag_id),
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Goal Instances table (with updated_at and deleted_at from the start)
CREATE TABLE IF NOT EXISTS goal_instances (
    id TEXT PRIMARY KEY,
    goal_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_goal_period ON goal_instances(goal_id, period_start);
CREATE INDEX IF NOT EXISTS idx_goal_instances_goal_id ON goal_instances(goal_id);

-- Goal Instance-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS goal_instance_tags (
    goal_instance_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (goal_instance_id, tag_id),
    FOREIGN KEY (goal_instance_id) REFERENCES goal_instances(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    is_completed INTEGER NOT NULL DEFAULT 0,
    due_date TEXT,
    goal_instance_id TEXT,
    goal_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (goal_instance_id) REFERENCES goal_instances(id) ON DELETE SET NULL,
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_deleted_at ON tasks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_goal_instance_id ON tasks(goal_instance_id);
CREATE INDEX IF NOT EXISTS idx_tasks_goal_id ON tasks(goal_id);

-- Task-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Subtasks table
CREATE TABLE IF NOT EXISTS subtasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    is_completed INTEGER NOT NULL DEFAULT 0,
    task_id TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_subtasks_deleted_at ON subtasks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_subtasks_task_id ON subtasks(task_id);
CREATE INDEX IF NOT EXISTS idx_subtasks_order_index ON subtasks(task_id, order_index);

-- Activities table (for activity tracking/audit logging)
CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities(created_at);
CREATE INDEX IF NOT EXISTS idx_activities_entity ON activities(entity_type, entity_id);

-- Bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    description TEXT,
    image_url TEXT,
    favicon_url TEXT,
    site_name TEXT,
    author TEXT,
    published_at TEXT,
    content_type TEXT, -- 'article', 'video', 'tweet', 'image', etc.
    metadata_json TEXT, -- Full metadata as JSON for extensibility
    is_archived INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url);
CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);
CREATE INDEX IF NOT EXISTS idx_bookmarks_deleted_at ON bookmarks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_bookmarks_content_type ON bookmarks(content_type);

-- Bookmark-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS bookmark_tags (
    bookmark_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (bookmark_id, tag_id),
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Canvases table
CREATE TABLE IF NOT EXISTS canvases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    canvas_data TEXT NOT NULL, -- JSON Canvas format
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_canvases_created_at ON canvases(created_at);
CREATE INDEX IF NOT EXISTS idx_canvases_updated_at ON canvases(updated_at);
CREATE INDEX IF NOT EXISTS idx_canvases_deleted_at ON canvases(deleted_at);

-- Resource links table (bidirectional linking between all resources)
CREATE TABLE IF NOT EXISTS resource_links (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL, -- 'entry', 'task', 'goal', 'canvas', 'bookmark'
    source_id TEXT NOT NULL,
    target_type TEXT NOT NULL, -- 'entry', 'task', 'goal', 'canvas', 'bookmark'
    target_id TEXT NOT NULL,
    link_text TEXT, -- Optional display text for the link
    created_at TEXT NOT NULL,
    UNIQUE(source_type, source_id, target_type, target_id)
);

CREATE INDEX IF NOT EXISTS idx_resource_links_source ON resource_links(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_resource_links_target ON resource_links(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_resource_links_created_at ON resource_links(created_at);
