-- Add updated_at and deleted_at columns to goal_instances table
ALTER TABLE goal_instances ADD COLUMN updated_at TEXT;
ALTER TABLE goal_instances ADD COLUMN deleted_at TEXT;

-- Update existing rows to set updated_at to created_at
UPDATE goal_instances SET updated_at = created_at WHERE updated_at IS NULL;
