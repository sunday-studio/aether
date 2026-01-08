package db

import (
	"aether/internal/logging"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/glebarez/sqlite"
	"github.com/tursodatabase/go-libsql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

func Initialize() (*gorm.DB, error) {
	gormLogger := logger.Default.LogMode(logger.Info)

	libsqlURL := os.Getenv("LIBSQL_URL")

	var db *gorm.DB
	var err error

	if libsqlURL != "" {
		log := logging.NewLogger()
		useReplica := os.Getenv("LIBSQL_USE_REPLICA") == "true"

		if useReplica {
			log.Info("Using libSQL embedded replica mode", "syncUrl", libsqlURL)

			replicaPath := "./libsql-replica/local.db"

			// Create replica directory
			if err := os.MkdirAll(filepath.Dir(replicaPath), 0755); err != nil {
				return nil, fmt.Errorf("failed to create replica dir: %w", err)
			}

			// Create embedded replica connector (no auth needed)
			connector, err := libsql.NewEmbeddedReplicaConnector(replicaPath, libsqlURL)
			if err != nil {
				return nil, fmt.Errorf("failed to create embedded replica connector: %w", err)
			}

			// Open database with connector
			sqlDB := sql.OpenDB(connector)

			// Normal connection pool settings
			sqlDB.SetMaxIdleConns(10)
			sqlDB.SetMaxOpenConns(50)
			sqlDB.SetConnMaxLifetime(time.Hour)

			// Perform initial sync
			log.Info("Performing initial sync with primary...")
			syncResult, err := connector.Sync()
			if err != nil {
				log.Warn("Initial sync failed (this is OK if primary is empty)", "error", err)
			} else {
				log.Info("Initial sync completed", "framesSynced", syncResult.FramesSynced)
			}

			db, err = gorm.Open(sqlite.Dialector{Conn: sqlDB}, &gorm.Config{
				Logger: gormLogger,
			})
			if err != nil {
				connector.Close()
				return nil, fmt.Errorf("failed to open GORM with libSQL replica: %w", err)
			}

			log.Info("Successfully connected to libSQL embedded replica")

			// Start periodic sync in background
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
			// Direct HTTP connection
			log.Info("Using libSQL HTTP connection", "url", libsqlURL)
			log.Warn("Direct HTTP mode may experience STREAM_EXPIRED errors. Consider setting LIBSQL_USE_REPLICA=true")

			sqlDB, err := sql.Open("libsql", libsqlURL)
			if err != nil {
				return nil, fmt.Errorf("failed to open libSQL connection: %w", err)
			}

			sqlDB.SetMaxIdleConns(0)
			sqlDB.SetMaxOpenConns(5)
			sqlDB.SetConnMaxLifetime(10 * time.Second)

			db, err = gorm.Open(sqlite.Dialector{Conn: sqlDB}, &gorm.Config{
				Logger:      gormLogger,
				PrepareStmt: false,
			})
			if err != nil {
				return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
			}

			log.Info("Connected to libSQL HTTP mode with aggressive pooling")
		}
	} else {
		// Fall back to local SQLite
		log := logging.NewLogger()
		log.Info("Using local SQLite database (LIBSQL_URL not set)")

		dataDir := "sqlite"
		if err := os.MkdirAll(dataDir, 0755); err != nil {
			return nil, err
		}

		dbPath := filepath.Join(dataDir, "aether.db")
		db, err = gorm.Open(sqlite.Open(dbPath), &gorm.Config{
			Logger: gormLogger,
		})
		if err != nil {
			return nil, err
		}

		sqlDB, err := db.DB()
		if err != nil {
			return nil, err
		}
		sqlDB.SetMaxIdleConns(10)
		sqlDB.SetMaxOpenConns(100)
	}

	return db, nil
}

func Migrate(db *gorm.DB) error {
	log := logging.NewLogger()

	// Check if we're using libSQL replica mode
	useReplica := os.Getenv("LIBSQL_USE_REPLICA") == "true"
	libsqlURL := os.Getenv("LIBSQL_URL")

	if useReplica {
		log.Info("Running migrations in libSQL replica mode")
		log.Info("Writes will be delegated to primary database")

		// First, check if tables already exist by trying to query them
		var taskCount int64
		tableExists := db.Model(&Task{}).Count(&taskCount).Error == nil

		if tableExists {
			log.Info("Tables already exist, skipping AutoMigrate")
			log.Info("If you need to update schema, run migrations on primary database first")
			return nil
		}

		log.Warn("Tables do not exist. Attempting to create them on primary database...")
		log.Warn("If this fails, you may need to run migrations directly on the primary database.")
	}

	log.Info("Running database migrations")

	models := []interface{}{
		&Entry{},
		&Tag{},
		&Task{},
		&Goal{},
		&GoalInstance{},
		&Settings{},
	}

	migrationErrors := []error{}
	for _, model := range models {
		if err := db.AutoMigrate(model); err != nil {
			log.Error("Failed to migrate model", "error", err, "model", fmt.Sprintf("%T", model))
			migrationErrors = append(migrationErrors, fmt.Errorf("%T: %w", model, err))

			// Don't fail immediately, try to migrate other models
			continue
		}
		log.Info("Successfully migrated model", "model", fmt.Sprintf("%T", model))
	}

	// If we had errors, provide helpful guidance
	if len(migrationErrors) > 0 {
		if useReplica {
			log.Error("=" + strings.Repeat("=", 70))
			log.Error("MIGRATION FAILED: Tables could not be created on primary database")
			log.Error("=" + strings.Repeat("=", 70))
			log.Error("")
			log.Error("SOLUTION: Run migrations directly on the primary database first:")
			log.Error("")
			log.Error("  1. Temporarily disable replica mode:")
			log.Error("     unset LIBSQL_USE_REPLICA")
			log.Error("")
			log.Error("  2. Or set it to false:")
			log.Error("     export LIBSQL_USE_REPLICA=false")
			log.Error("")
			if libsqlURL != "" {
				log.Error(fmt.Sprintf("  3. Ensure LIBSQL_URL is set: %s", libsqlURL))
			}
			log.Error("")
			log.Error("  4. Restart the application to run migrations on primary")
			log.Error("")
			log.Error("  5. After migrations succeed, re-enable replica mode:")
			log.Error("     export LIBSQL_USE_REPLICA=true")
			log.Error("")
			log.Error("=" + strings.Repeat("=", 70))
		}

		return fmt.Errorf("failed to migrate %d model(s): %v", len(migrationErrors), migrationErrors)
	}

	// Verify tables exist by checking if we can query them
	var count int64
	if err := db.Model(&Task{}).Count(&count).Error; err != nil {
		log.Warn("Could not verify tasks table exists", "error", err)
		if useReplica {
			log.Error("Table verification failed. Primary database may not have tables.")
		}
	} else {
		log.Info("Verified tasks table exists and is accessible", "rowCount", count)
	}

	log.Info("All database migrations completed successfully")
	return nil
}
