package migrations

import (
	"aether/internal/logging"
	"fmt"

	"gorm.io/gorm"
)

func init() {
	Register(Migration{
		Version: "20260109_203215_add_is_non_recurring_to_goals",
		Name:    "Add is_non_recurring field to goals and make recurrence fields nullable",
		Up: func(tx *gorm.DB) error {
			log := logging.NewLogger()
			log.Info("Running migration: Add is_non_recurring field to goals and make recurrence fields nullable")

			// Disable foreign key constraints for this migration
			if err := tx.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
				return fmt.Errorf("failed to disable foreign keys: %w", err)
			}
			log.Info("Disabled foreign key constraints")

			// Add is_non_recurring column to goals table (default: false)
			// Check if column already exists first
			var count int
			tx.Raw("SELECT COUNT(*) FROM pragma_table_info('goals') WHERE name='is_non_recurring'").Scan(&count)
			if count == 0 {
				if err := tx.Exec("ALTER TABLE goals ADD COLUMN is_non_recurring BOOLEAN NOT NULL DEFAULT 0").Error; err != nil {
					return fmt.Errorf("failed to add is_non_recurring column: %w", err)
				}
				log.Info("Added is_non_recurring column to goals table")
			} else {
				log.Info("is_non_recurring column already exists, skipping")
			}

			// For SQLite, we can't directly alter NOT NULL constraints.
			// GORM AutoMigrate will handle the schema changes when models are updated.
			// The model changes (using pointers) will allow NULL values going forward.
			// Existing data will remain valid, and new NULL values will be accepted.
			log.Info("Note: Making columns nullable requires model changes - GORM AutoMigrate will handle this")

			// Re-enable foreign key constraints
			if err := tx.Exec("PRAGMA foreign_keys = ON").Error; err != nil {
				return fmt.Errorf("failed to re-enable foreign keys: %w", err)
			}
			log.Info("Re-enabled foreign key constraints")

			log.Info("Migration completed successfully")
			return nil
		},
		Down: func(tx *gorm.DB) error {
			log := logging.NewLogger()
			log.Info("Rolling back migration: Add is_non_recurring field to goals and make recurrence fields nullable")

			// Disable foreign key constraints
			if err := tx.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
				return fmt.Errorf("failed to disable foreign keys: %w", err)
			}

			// Remove is_non_recurring column (SQLite doesn't support DROP COLUMN directly in older versions)
			// For SQLite 3.35.0+, we can use ALTER TABLE DROP COLUMN
			// For older versions, we'd need to recreate the table, which is complex
			// For now, we'll just update NULL values to defaults
			if err := tx.Exec("UPDATE goals SET recurrence_type = '' WHERE recurrence_type IS NULL").Error; err != nil {
				log.Warn("Failed to update NULL recurrence_type values", "error", err)
			}
			if err := tx.Exec("UPDATE goals SET recurrence_interval = 1 WHERE recurrence_interval IS NULL").Error; err != nil {
				log.Warn("Failed to update NULL recurrence_interval values", "error", err)
			}
			if err := tx.Exec("UPDATE goal_instances SET period_end = period_start WHERE period_end IS NULL").Error; err != nil {
				log.Warn("Failed to update NULL period_end values", "error", err)
			}

			// Try to drop the column (will fail on older SQLite versions)
			if err := tx.Exec("ALTER TABLE goals DROP COLUMN is_non_recurring").Error; err != nil {
				log.Warn("Could not drop is_non_recurring column (may require SQLite 3.35.0+)", "error", err)
			} else {
				log.Info("Removed is_non_recurring column")
			}

			// Re-enable foreign key constraints
			if err := tx.Exec("PRAGMA foreign_keys = ON").Error; err != nil {
				return fmt.Errorf("failed to re-enable foreign keys: %w", err)
			}

			log.Info("Rollback completed")
			return nil
		},
	})
}
