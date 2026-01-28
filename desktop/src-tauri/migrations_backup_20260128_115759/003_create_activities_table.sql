-- Create activities table for activity tracking and audit logging
CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    metadata TEXT
);

-- Index on created_at for efficient date-based queries
CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities(created_at);

-- Index on entity_type and entity_id for audit log lookups
CREATE INDEX IF NOT EXISTS idx_activities_entity ON activities(entity_type, entity_id);
