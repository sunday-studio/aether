package migrations

import (
	"aether/internal/logging"
	"fmt"
	"strings"

	"gorm.io/gorm"
)

func init() {
	Register(Migration{
		Version:       "20260108_154500_update_id_separator_to_underscore",
		Name:          "Update ID separator from hyphen to underscore",
		NoTransaction: true, // Need to disable foreign keys which can't be done in a transaction
		Up: func(tx *gorm.DB) error {
			log := logging.NewLogger()
			log.Info("Running migration: Update ID separator from hyphen to underscore")

			// Disable foreign key constraints for this migration
			if err := tx.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
				return fmt.Errorf("failed to disable foreign keys: %w", err)
			}
			log.Info("Disabled foreign key constraints")

			// Define tables and their ID prefixes
			type tableInfo struct {
				tableName string
				prefix    string
			}

			tables := []tableInfo{
				{"tasks", "task"},
				{"goals", "goal"},
				{"tags", "tag"},
				{"entries", "entry"},
				{"goal_instances", "goal-instance"},
			}

			totalUpdated := 0

			for _, ti := range tables {
				// Find all records with hyphen separator (old format: prefix-uuid)
				var records []struct {
					ID string
				}

				// Select all IDs that start with prefix and have hyphen separator
				query := fmt.Sprintf("SELECT id FROM %s WHERE id LIKE ?", ti.tableName)
				pattern := ti.prefix + "-%"

				if err := tx.Raw(query, pattern).Scan(&records).Error; err != nil {
					log.Warn("Could not query table", "table", ti.tableName, "error", err)
					continue
				}

				if len(records) == 0 {
					log.Info("No records to update", "table", ti.tableName)
					continue
				}

				log.Info("Found records to update", "table", ti.tableName, "count", len(records))

				// Update each record
				for _, record := range records {
					oldID := record.ID

					// Replace only the first hyphen (the separator) with underscore
					// Keep the UUID part intact (which also has hyphens)
					newID := strings.Replace(oldID, ti.prefix+"-", ti.prefix+"_", 1)

					// Update the ID
					updateQuery := fmt.Sprintf("UPDATE %s SET id = ? WHERE id = ?", ti.tableName)
					result := tx.Exec(updateQuery, newID, oldID)

					if result.Error != nil {
						return fmt.Errorf("failed to update ID in %s: %w", ti.tableName, result.Error)
					}

					totalUpdated++
				}

				log.Info("Updated IDs", "table", ti.tableName, "count", len(records))
			}

			// Update foreign key references
			log.Info("Updating foreign key references")

			// Update goal_instance_id in tasks table
			updateFK := `
				UPDATE tasks 
				SET goal_instance_id = REPLACE(goal_instance_id, 'goal-instance-', 'goal-instance_')
				WHERE goal_instance_id LIKE 'goal-instance-%'
			`
			if result := tx.Exec(updateFK); result.Error != nil {
				log.Warn("Failed to update goal_instance_id foreign keys", "error", result.Error)
			} else if result.RowsAffected > 0 {
				log.Info("Updated goal_instance_id foreign keys", "count", result.RowsAffected)
			}

			// Update goal_id in goal_instances table
			updateGoalFK := `
				UPDATE goal_instances 
				SET goal_id = REPLACE(goal_id, 'goal-', 'goal_')
				WHERE goal_id LIKE 'goal-%'
			`
			if result := tx.Exec(updateGoalFK); result.Error != nil {
				log.Warn("Failed to update goal_id foreign keys", "error", result.Error)
			} else if result.RowsAffected > 0 {
				log.Info("Updated goal_id foreign keys", "count", result.RowsAffected)
			}

			// Update junction tables for many-to-many relationships
			junctionTables := []struct {
				table  string
				column string
				prefix string
			}{
				{"entry_tags", "entry_id", "entry"},
				{"entry_tags", "tag_id", "tag"},
				{"task_tags", "task_id", "task"},
				{"task_tags", "tag_id", "tag"},
				{"goal_tags", "goal_id", "goal"},
				{"goal_tags", "tag_id", "tag"},
				{"goal_instance_tags", "goal_instance_id", "goal-instance"},
				{"goal_instance_tags", "tag_id", "tag"},
			}

			for _, jt := range junctionTables {
				updateJunction := fmt.Sprintf(`
					UPDATE %s 
					SET %s = REPLACE(%s, '%s-', '%s_')
					WHERE %s LIKE '%s-%%'
				`, jt.table, jt.column, jt.column, jt.prefix, jt.prefix, jt.column, jt.prefix)

				if result := tx.Exec(updateJunction); result.Error != nil {
					log.Warn("Failed to update junction table", "table", jt.table, "column", jt.column, "error", result.Error)
				} else if result.RowsAffected > 0 {
					log.Info("Updated junction table", "table", jt.table, "column", jt.column, "count", result.RowsAffected)
				}
			}

			log.Info("Migration completed", "totalPrimaryKeysUpdated", totalUpdated)

			// Re-enable foreign key constraints
			if err := tx.Exec("PRAGMA foreign_keys = ON").Error; err != nil {
				return fmt.Errorf("failed to re-enable foreign keys: %w", err)
			}
			log.Info("Re-enabled foreign key constraints")

			return nil
		},
		Down: func(tx *gorm.DB) error {
			log := logging.NewLogger()
			log.Info("Rolling back migration: Update ID separator from hyphen to underscore")

			// Disable foreign key constraints for rollback
			if err := tx.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
				return fmt.Errorf("failed to disable foreign keys: %w", err)
			}
			log.Info("Disabled foreign key constraints")

			// Reverse the migration: change underscores back to hyphens
			type tableInfo struct {
				tableName string
				prefix    string
			}

			tables := []tableInfo{
				{"tasks", "task"},
				{"goals", "goal"},
				{"tags", "tag"},
				{"entries", "entry"},
				{"goal_instances", "goal-instance"},
			}

			for _, ti := range tables {
				var records []struct {
					ID string
				}

				query := fmt.Sprintf("SELECT id FROM %s WHERE id LIKE ?", ti.tableName)
				pattern := ti.prefix + "_%"

				if err := tx.Raw(query, pattern).Scan(&records).Error; err != nil {
					log.Warn("Could not query table", "table", ti.tableName, "error", err)
					continue
				}

				for _, record := range records {
					oldID := record.ID
					newID := strings.Replace(oldID, ti.prefix+"_", ti.prefix+"-", 1)

					updateQuery := fmt.Sprintf("UPDATE %s SET id = ? WHERE id = ?", ti.tableName)
					if result := tx.Exec(updateQuery, newID, oldID); result.Error != nil {
						return fmt.Errorf("failed to rollback ID in %s: %w", ti.tableName, result.Error)
					}
				}

				log.Info("Rolled back IDs", "table", ti.tableName, "count", len(records))
			}

			// Rollback foreign key references
			rollbackFK := `
				UPDATE tasks 
				SET goal_instance_id = REPLACE(goal_instance_id, 'goal-instance_', 'goal-instance-')
				WHERE goal_instance_id LIKE 'goal-instance_%'
			`
			tx.Exec(rollbackFK)

			rollbackGoalFK := `
				UPDATE goal_instances 
				SET goal_id = REPLACE(goal_id, 'goal_', 'goal-')
				WHERE goal_id LIKE 'goal_%'
			`
			tx.Exec(rollbackGoalFK)

			// Rollback junction tables
			junctionTables := []struct {
				table  string
				column string
				prefix string
			}{
				{"entry_tags", "entry_id", "entry"},
				{"entry_tags", "tag_id", "tag"},
				{"task_tags", "task_id", "task"},
				{"task_tags", "tag_id", "tag"},
				{"goal_tags", "goal_id", "goal"},
				{"goal_tags", "tag_id", "tag"},
				{"goal_instance_tags", "goal_instance_id", "goal-instance"},
				{"goal_instance_tags", "tag_id", "tag"},
			}

			for _, jt := range junctionTables {
				rollbackJunction := fmt.Sprintf(`
					UPDATE %s 
					SET %s = REPLACE(%s, '%s_', '%s-')
					WHERE %s LIKE '%s_%%'
				`, jt.table, jt.column, jt.column, jt.prefix, jt.prefix, jt.column, jt.prefix)

				tx.Exec(rollbackJunction)
			}

			log.Info("Rollback completed")

			// Re-enable foreign key constraints
			if err := tx.Exec("PRAGMA foreign_keys = ON").Error; err != nil {
				return fmt.Errorf("failed to re-enable foreign keys: %w", err)
			}
			log.Info("Re-enabled foreign key constraints")

			return nil
		},
	})
}
