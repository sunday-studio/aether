package db

import (
	"aether/internal/logging"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/glebarez/sqlite"
	"github.com/tursodatabase/go-libsql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

func Initialize() (*gorm.DB, error) {
	gormLogger := logger.Default.LogMode(logger.Info)

	libsqlURL := os.Getenv("LIBSQL_URL")
	if libsqlURL == "" {
		return nil, fmt.Errorf("LIBSQL_URL environment variable must be set")
	}

	log := logging.NewLogger()
	useReplica := os.Getenv("LIBSQL_USE_REPLICA") == "true"

	var db *gorm.DB
	var err error

	if useReplica {
		log.Info("Using libSQL embedded replica mode", "syncUrl", libsqlURL)

		replicaPath := "./libsql-replica/local.db"

		if err = os.MkdirAll(filepath.Dir(replicaPath), 0755); err != nil {
			return nil, fmt.Errorf("failed to create replica dir: %w", err)
		}

		connector, err := libsql.NewEmbeddedReplicaConnector(replicaPath, libsqlURL)
		if err != nil {
			return nil, fmt.Errorf("failed to create embedded replica connector: %w", err)
		}

		sqlDB := sql.OpenDB(connector)
		sqlDB.SetMaxIdleConns(10)
		sqlDB.SetMaxOpenConns(50)
		sqlDB.SetConnMaxLifetime(time.Hour)

		log.Info("Performing initial sync with primary...")
		syncResult, err := connector.Sync()
		if err != nil {
			log.Warn("Initial sync failed (this is OK if primary is empty)", "error", err)
		} else {
			log.Info("Initial sync completed", "framesSynced", syncResult.FramesSynced)
		}

		// Use glebarez/sqlite dialector with the custom connection
		db, err = gorm.Open(sqlite.Dialector{
			Conn: sqlDB,
		}, &gorm.Config{
			Logger: gormLogger,
		})
		if err != nil {
			connector.Close()
			return nil, fmt.Errorf("failed to open GORM with libSQL replica: %w", err)
		}

		log.Info("Successfully connected to libSQL embedded replica")

		syncInterval := 5 * time.Second
		if intervalEnv := os.Getenv("LIBSQL_SYNC_INTERVAL"); intervalEnv != "" {
			if duration, err := time.ParseDuration(intervalEnv + "s"); err == nil {
				syncInterval = duration
			}
		}

		go func() {
			ticker := time.NewTicker(syncInterval)
			defer ticker.Stop()

			for range ticker.C {
				syncResult, err := connector.Sync()
				if err != nil {
					log.Warn("Background sync failed", "error", err)
				} else if syncResult.FramesSynced > 0 {
					log.Info("Background sync completed", "framesSynced", syncResult.FramesSynced)
				}
			}
		}()

	} else {
		log.Info("Using libSQL HTTP connection", "url", libsqlURL)
		log.Warn("Direct HTTP mode may experience STREAM_EXPIRED errors. Consider setting LIBSQL_USE_REPLICA=true")

		sqlDB, err := sql.Open("libsql", libsqlURL)
		if err != nil {
			return nil, fmt.Errorf("failed to open libSQL connection: %w", err)
		}

		sqlDB.SetMaxIdleConns(0)
		sqlDB.SetMaxOpenConns(5)
		sqlDB.SetConnMaxLifetime(10 * time.Second)

		db, err = gorm.Open(sqlite.Dialector{
			Conn: sqlDB,
		}, &gorm.Config{
			Logger:      gormLogger,
			PrepareStmt: false,
		})
		if err != nil {
			return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
		}

		log.Info("Connected to libSQL HTTP mode with aggressive pooling")
	}

	return db, nil
}

func Migrate(db *gorm.DB) error {
	log := logging.NewLogger()

	log.Info("Running database migrations")

	// Temporarily disable foreign key constraints to allow schema changes
	// This is safe because AutoMigrate only adds columns/indexes, doesn't drop data
	if err := db.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
		log.Warn("Could not disable foreign keys (may not be supported)", "error", err)
		// Continue anyway - might work without disabling
	}

	// AutoMigrate only adds missing columns/indexes, doesn't drop tables
	// Order matters: migrate tables without foreign keys first
	models := []interface{}{
		&Entry{},
		&Tag{},
		&Settings{},
		&Goal{},
		&GoalInstance{},
		&Task{},
		&SubTask{},
	}

	for _, model := range models {
		if err := db.AutoMigrate(model); err != nil {
			log.Error("Failed to migrate model", "error", err, "model", fmt.Sprintf("%T", model))
			// Re-enable foreign keys before returning error
			db.Exec("PRAGMA foreign_keys = ON")
			return fmt.Errorf("failed to migrate %T: %w", model, err)
		}
		log.Info("Successfully migrated model", "model", fmt.Sprintf("%T", model))
	}

	// Re-enable foreign key constraints
	if err := db.Exec("PRAGMA foreign_keys = ON").Error; err != nil {
		log.Warn("Could not re-enable foreign keys", "error", err)
	}

	log.Info("All database migrations completed successfully")
	return nil
}
