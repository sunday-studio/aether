// package db

// import (
// 	"aether/internal/logging"
// 	"database/sql"
// 	"fmt"
// 	"net/url"
// 	"os"
// 	"path/filepath"
// 	"time"

// 	"github.com/glebarez/sqlite"
// 	_ "github.com/tursodatabase/go-libsql"
// 	"gorm.io/gorm"
// 	"gorm.io/gorm/logger"
// )

// func Initialize() (*gorm.DB, error) {
// 	gormLogger := logger.Default.LogMode(logger.Info)

// 	libsqlURL := os.Getenv("LIBSQL_URL")
// 	libsqlAuthToken := os.Getenv("LIBSQL_AUTH_TOKEN")

// 	var db *gorm.DB
// 	var err error

// 	if libsqlURL != "" {
// 		log := logging.NewLogger()

// 		// Check if we should use embedded replica mode
// 		useReplica := os.Getenv("LIBSQL_USE_REPLICA") == "true"

// 		if useReplica {
// 			log.Info("Using libSQL embedded replica mode", "syncUrl", libsqlURL)

// 			// Embedded replica: local file that syncs from remote server
// 			replicaPath := "./libsql-replica/local.db"

// 			// Create replica directory
// 			if err := os.MkdirAll(filepath.Dir(replicaPath), 0755); err != nil {
// 				return nil, fmt.Errorf("failed to create replica dir: %w", err)
// 			}

// 			// Build connection string for embedded replica
// 			// Format: file:path?sync=url&authToken=token&syncInterval=5
// 			u, err := url.Parse(fmt.Sprintf("file:%s", replicaPath))
// 			if err != nil {
// 				return nil, fmt.Errorf("invalid replica path: %w", err)
// 			}

// 			q := u.Query()
// 			q.Set("sync", libsqlURL)
// 			if libsqlAuthToken != "" {
// 				q.Set("authToken", libsqlAuthToken)
// 			}
// 			// Sync interval in seconds (default: 60)
// 			q.Set("syncInterval", "5") // Sync every 5 seconds
// 			u.RawQuery = q.Encode()

// 			connURL := u.String()
// 			log.Info("Embedded replica connection string", "url", connURL)

// 			sqlDB, err := sql.Open("libsql", connURL)
// 			if err != nil {
// 				return nil, fmt.Errorf("failed to open libSQL replica: %w", err)
// 			}

// 			// With file-based replica, normal pooling works fine
// 			sqlDB.SetMaxIdleConns(10)
// 			sqlDB.SetMaxOpenConns(50)
// 			sqlDB.SetConnMaxLifetime(time.Hour)

// 			db, err = gorm.Open(sqlite.Dialector{Conn: sqlDB}, &gorm.Config{
// 				Logger: gormLogger,
// 			})
// 			if err != nil {
// 				return nil, fmt.Errorf("failed to open GORM with libSQL replica: %w", err)
// 			}

// 			log.Info("Successfully connected to libSQL embedded replica")
// 		} else {
// 			// Direct HTTP connection (fallback)
// 			log.Info("Using libSQL HTTP connection", "url", libsqlURL)
// 			log.Warn("Direct HTTP mode may experience STREAM_EXPIRED errors. Consider setting LIBSQL_USE_REPLICA=true")

// 			connURL := libsqlURL
// 			if libsqlAuthToken != "" {
// 				u, err := url.Parse(libsqlURL)
// 				if err != nil {
// 					return nil, fmt.Errorf("invalid libSQL URL: %w", err)
// 				}
// 				q := u.Query()
// 				q.Set("authToken", libsqlAuthToken)
// 				u.RawQuery = q.Encode()
// 				connURL = u.String()
// 			}

// 			sqlDB, err := sql.Open("libsql", connURL)
// 			if err != nil {
// 				return nil, fmt.Errorf("failed to open libSQL connection: %w", err)
// 			}

// 			// Aggressive settings for HTTP to prevent stream expiration
// 			sqlDB.SetMaxIdleConns(0)
// 			sqlDB.SetMaxOpenConns(5)
// 			sqlDB.SetConnMaxLifetime(10 * time.Second)

// 			db, err = gorm.Open(sqlite.Dialector{Conn: sqlDB}, &gorm.Config{
// 				Logger:      gormLogger,
// 				PrepareStmt: false,
// 			})
// 			if err != nil {
// 				return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
// 			}

// 			log.Info("Connected to libSQL HTTP mode with aggressive pooling")
// 		}
// 	} else {
// 		// Fall back to local SQLite for development
// 		log := logging.NewLogger()
// 		log.Info("Using local SQLite database (LIBSQL_URL not set)")

// 		dataDir := "sqlite"
// 		if err := os.MkdirAll(dataDir, 0755); err != nil {
// 			return nil, err
// 		}

// 		dbPath := filepath.Join(dataDir, "aether.db")
// 		db, err = gorm.Open(sqlite.Open(dbPath), &gorm.Config{
// 			Logger: gormLogger,
// 		})
// 		if err != nil {
// 			return nil, err
// 		}

// 		sqlDB, err := db.DB()
// 		if err != nil {
// 			return nil, err
// 		}
// 		sqlDB.SetMaxIdleConns(10)
// 		sqlDB.SetMaxOpenConns(100)
// 	}

// 	return db, nil
// }

// func Migrate(db *gorm.DB) error {
// 	log := logging.NewLogger()
// 	log.Info("Running database migrations")
// 	return db.AutoMigrate(&Entry{}, &Tag{}, &Task{}, &Goal{}, &GoalInstance{})
// }

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
	log.Info("Running database migrations")
	return db.AutoMigrate(&Entry{}, &Tag{}, &Task{}, &Goal{}, &GoalInstance{})
}
