package migrations

import (
	"aether/internal/db"
	"aether/internal/logging"
	"fmt"
	"time"

	"gorm.io/gorm"
)

// RunPending executes all pending migrations
func RunCustomMigrations(database *gorm.DB) error {
	log := logging.NewLogger()

	// Ensure schema_migrations table exists
	if err := database.AutoMigrate(&db.SchemaMigration{}); err != nil {
		return fmt.Errorf("failed to create schema_migrations table: %w", err)
	}

	// Get all registered migrations
	allMigrations := GetAll()
	if len(allMigrations) == 0 {
		log.Info("No migrations registered")
		return nil
	}

	// Get already applied migrations
	var applied []db.SchemaMigration
	if err := database.Order("version ASC").Find(&applied).Error; err != nil {
		return fmt.Errorf("failed to fetch applied migrations: %w", err)
	}

	// Create map of applied versions for quick lookup
	appliedMap := make(map[string]bool)
	for _, m := range applied {
		appliedMap[m.Version] = true
	}

	// Run pending migrations
	pendingCount := 0
	for _, migration := range allMigrations {
		if appliedMap[migration.Version] {
			continue
		}

		pendingCount++
		log.Info("Running migration", "version", migration.Version, "name", migration.Name)

		var err error

		if migration.NoTransaction {
			// Run migration without transaction (needed for PRAGMA changes)
			log.Info("Running migration without transaction", "version", migration.Version)

			// Execute the migration
			if err = migration.Up(database); err != nil {
				log.Error("Migration failed", "version", migration.Version, "error", err)
				return fmt.Errorf("migration %s failed: %w", migration.Version, err)
			}

			// Record the migration
			record := db.SchemaMigration{
				Version:   migration.Version,
				Name:      migration.Name,
				AppliedAt: time.Now(),
			}
			if err = database.Create(&record).Error; err != nil {
				log.Error("Failed to record migration", "version", migration.Version, "error", err)
				return fmt.Errorf("failed to record migration %s: %w", migration.Version, err)
			}
		} else {
			// Run migration in a transaction
			err = database.Transaction(func(tx *gorm.DB) error {
				// Execute the migration
				if err := migration.Up(tx); err != nil {
					return fmt.Errorf("migration failed: %w", err)
				}

				// Record the migration
				record := db.SchemaMigration{
					Version:   migration.Version,
					Name:      migration.Name,
					AppliedAt: time.Now(),
				}
				if err := tx.Create(&record).Error; err != nil {
					return fmt.Errorf("failed to record migration: %w", err)
				}

				return nil
			})

			if err != nil {
				log.Error("Migration failed", "version", migration.Version, "error", err)
				return fmt.Errorf("migration %s failed: %w", migration.Version, err)
			}
		}

		log.Info("Migration completed", "version", migration.Version)
	}

	if pendingCount == 0 {
		log.Info("No pending migrations")
	} else {
		log.Info("All migrations completed", "count", pendingCount)
	}

	return nil
}

// Rollback rolls back the last N migrations
func Rollback(database *gorm.DB, steps int) error {
	log := logging.NewLogger()

	if steps <= 0 {
		return fmt.Errorf("steps must be positive")
	}

	// Get applied migrations in reverse order
	var applied []db.SchemaMigration
	if err := database.Order("version DESC").Limit(steps).Find(&applied).Error; err != nil {
		return fmt.Errorf("failed to fetch applied migrations: %w", err)
	}

	if len(applied) == 0 {
		log.Info("No migrations to rollback")
		return nil
	}

	// Rollback each migration
	for _, record := range applied {
		migration := GetByVersion(record.Version)
		if migration == nil {
			log.Warn("Migration not found in registry", "version", record.Version)
			continue
		}

		log.Info("Rolling back migration", "version", migration.Version, "name", migration.Name)

		var err error

		if migration.NoTransaction {
			// Run rollback without transaction (needed for PRAGMA changes)
			log.Info("Running rollback without transaction", "version", migration.Version)

			// Execute the rollback
			if err = migration.Down(database); err != nil {
				log.Error("Rollback failed", "version", migration.Version, "error", err)
				return fmt.Errorf("rollback of %s failed: %w", migration.Version, err)
			}

			// Remove the migration record
			if err = database.Delete(&record).Error; err != nil {
				log.Error("Failed to remove migration record", "version", migration.Version, "error", err)
				return fmt.Errorf("failed to remove migration record for %s: %w", migration.Version, err)
			}
		} else {
			// Run rollback in a transaction
			err = database.Transaction(func(tx *gorm.DB) error {
				// Execute the rollback
				if err := migration.Down(tx); err != nil {
					return fmt.Errorf("rollback failed: %w", err)
				}

				// Remove the migration record
				if err := tx.Delete(&record).Error; err != nil {
					return fmt.Errorf("failed to remove migration record: %w", err)
				}

				return nil
			})

			if err != nil {
				log.Error("Rollback failed", "version", migration.Version, "error", err)
				return fmt.Errorf("rollback of %s failed: %w", migration.Version, err)
			}
		}

		log.Info("Rollback completed", "version", migration.Version)
	}

	log.Info("Rollback completed", "count", len(applied))
	return nil
}

// GetStatus returns the status of all migrations
func GetStatus(database *gorm.DB) ([]MigrationStatus, error) {
	// Get all registered migrations
	allMigrations := GetAll()

	// Get applied migrations
	var applied []db.SchemaMigration
	if err := database.Order("version ASC").Find(&applied).Error; err != nil {
		return nil, fmt.Errorf("failed to fetch applied migrations: %w", err)
	}

	// Create map of applied versions
	appliedMap := make(map[string]db.SchemaMigration)
	for _, m := range applied {
		appliedMap[m.Version] = m
	}

	// Build status list
	var statuses []MigrationStatus
	for _, migration := range allMigrations {
		status := MigrationStatus{
			Version: migration.Version,
			Name:    migration.Name,
		}

		if record, ok := appliedMap[migration.Version]; ok {
			status.Applied = true
			status.AppliedAt = &record.AppliedAt
		} else {
			status.Applied = false
		}

		statuses = append(statuses, status)
	}

	return statuses, nil
}

// MigrationStatus represents the status of a migration
type MigrationStatus struct {
	Version   string
	Name      string
	Applied   bool
	AppliedAt *time.Time
}
