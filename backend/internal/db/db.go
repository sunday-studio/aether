// package db

// import (
// 	"aether/internal/logging"
// 	"database/sql"
// 	"fmt"
// 	"os"
// 	"path/filepath"
// 	"sync"
// 	"time"

// 	"github.com/glebarez/sqlite"
// 	"github.com/tursodatabase/go-libsql"
// 	"gorm.io/gorm"
// 	"gorm.io/gorm/logger"
// )

// var (
// 	// Global connector for manual sync access
// 	globalConnector *libsql.Connector
// 	connectorMutex  sync.RWMutex
// )

// func Initialize() (*gorm.DB, error) {
// 	gormLogger := logger.Default.LogMode(logger.Info)

// 	libsqlURL := os.Getenv("LIBSQL_URL")
// 	if libsqlURL == "" {
// 		return nil, fmt.Errorf("LIBSQL_URL environment variable must be set")
// 	}

// 	authToken := os.Getenv("LIBSQL_AUTH_TOKEN")
// 	if authToken == "" {
// 		return nil, fmt.Errorf("LIBSQL_AUTH_TOKEN environment variable must be set")
// 	}

// 	log := logging.NewLogger()
// 	useReplica := os.Getenv("LIBSQL_USE_REPLICA") == "true"

// 	fmt.Println("useReplica ->", useReplica)

// 	var db *gorm.DB
// 	var err error

// 	if useReplica {
// 		log.Info("Using libSQL embedded replica mode", "syncUrl", libsqlURL)

// 		replicaPath := "./libsql-replica/local.db"

// 		if err = os.MkdirAll(filepath.Dir(replicaPath), 0755); err != nil {
// 			return nil, fmt.Errorf("failed to create replica dir: %w", err)
// 		}

// 		authToken := os.Getenv("LIBSQL_AUTH_TOKEN")

// 		connector, err := libsql.NewEmbeddedReplicaConnector(replicaPath, libsqlURL,
// 			libsql.WithAuthToken(authToken))
// 		if err != nil {
// 			return nil, fmt.Errorf("failed to create embedded replica connector: %w", err)
// 		}

// 		// Store connector globally for manual sync access
// 		connectorMutex.Lock()
// 		globalConnector = connector
// 		connectorMutex.Unlock()

// 		sqlDB := sql.OpenDB(connector)
// 		sqlDB.SetMaxIdleConns(10)
// 		sqlDB.SetMaxOpenConns(50)
// 		sqlDB.SetConnMaxLifetime(time.Hour)

// 		log.Info("Performing initial sync with primary...")
// 		syncResult, err := connector.Sync()
// 		if err != nil {
// 			log.Warn("Initial sync failed (this is OK if primary is empty)", "error", err)
// 		} else {
// 			log.Info("Initial sync completed", "framesSynced", syncResult.FramesSynced)
// 		}

// 		// Use glebarez/sqlite dialector with the custom connection
// 		db, err = gorm.Open(sqlite.Dialector{
// 			Conn: sqlDB,
// 		}, &gorm.Config{
// 			Logger:      gormLogger,
// 			PrepareStmt: true,
// 		})
// 		if err != nil {
// 			connector.Close()
// 			return nil, fmt.Errorf("failed to open GORM with libSQL replica: %w", err)
// 		}

// 		// Apply SQLite performance optimizations
// 		if err := applySQLiteOptimizations(db); err != nil {
// 			log.Warn("Failed to apply SQLite optimizations", "error", err)
// 		}

// 		log.Info("Successfully connected to libSQL embedded replica")
// 		log.Info("Automatic background sync disabled - use /v1/sync endpoint for manual sync")

// 	} else {
// 		log.Info("Using libSQL HTTP connection", "url", libsqlURL)
// 		log.Warn("Direct HTTP mode may experience STREAM_EXPIRED errors. Consider setting LIBSQL_USE_REPLICA=true")

// 		sqlDB, err := sql.Open("libsql", libsqlURL)
// 		if err != nil {
// 			return nil, fmt.Errorf("failed to open libSQL connection: %w", err)
// 		}

// 		// Improved connection pool settings
// 		sqlDB.SetMaxIdleConns(2)
// 		sqlDB.SetMaxOpenConns(10)
// 		sqlDB.SetConnMaxLifetime(30 * time.Second)

// 		db, err = gorm.Open(sqlite.Dialector{
// 			Conn: sqlDB,
// 		}, &gorm.Config{
// 			Logger:      gormLogger,
// 			PrepareStmt: true,
// 		})
// 		if err != nil {
// 			return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
// 		}

// 		log.Info("Connected to libSQL HTTP mode with improved pooling")
// 	}

// 	return db, nil
// }

// // applySQLiteOptimizations applies performance-enhancing PRAGMA settings
// func applySQLiteOptimizations(db *gorm.DB) error {
// 	pragmas := []string{
// 		"PRAGMA journal_mode = WAL",        // Write-Ahead Logging for better concurrency
// 		"PRAGMA synchronous = NORMAL",      // Balance between safety and performance
// 		"PRAGMA cache_size = -64000",       // 64MB cache (negative means KB)
// 		"PRAGMA temp_store = MEMORY",       // Store temp tables in memory
// 		"PRAGMA mmap_size = 268435456",     // 256MB memory-mapped I/O
// 		"PRAGMA page_size = 4096",          // Optimal page size
// 		"PRAGMA busy_timeout = 5000",       // Wait 5s if database is locked
// 		"PRAGMA foreign_keys = ON",         // Enable foreign key constraints
// 		"PRAGMA auto_vacuum = INCREMENTAL", // Prevent database bloat
// 	}

// 	for _, pragma := range pragmas {
// 		if err := db.Exec(pragma).Error; err != nil {
// 			return fmt.Errorf("failed to execute %s: %w", pragma, err)
// 		}
// 	}

// 	return nil
// }

// func Migrate(db *gorm.DB) error {
// 	log := logging.NewLogger()

// 	log.Info("Running database migrations")

// 	// Temporarily disable foreign key constraints to allow schema changes
// 	// This is safe because AutoMigrate only adds columns/indexes, doesn't drop data
// 	if err := db.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
// 		log.Warn("Could not disable foreign keys (may not be supported)", "error", err)
// 		// Continue anyway - might work without disabling
// 	}

// 	// AutoMigrate only adds missing columns/indexes, doesn't drop tables
// 	// Order matters: migrate tables without foreign keys first
// 	models := []interface{}{
// 		&Entry{},
// 		&Tag{},
// 		&Settings{},
// 		&Goal{},
// 		&GoalInstance{},
// 		&Task{},
// 		&SubTask{},
// 	}

// 	for _, model := range models {
// 		if err := db.AutoMigrate(model); err != nil {
// 			log.Error("Failed to migrate model", "error", err, "model", fmt.Sprintf("%T", model))
// 			// Re-enable foreign keys before returning error
// 			db.Exec("PRAGMA foreign_keys = ON")
// 			return fmt.Errorf("failed to migrate %T: %w", model, err)
// 		}
// 		log.Info("Successfully migrated model", "model", fmt.Sprintf("%T", model))
// 	}

// 	// Re-enable foreign key constraints
// 	if err := db.Exec("PRAGMA foreign_keys = ON").Error; err != nil {
// 		log.Warn("Could not re-enable foreign keys", "error", err)
// 	}

// 	log.Info("All database migrations completed successfully")
// 	return nil
// }

// // GetConnector returns the global libSQL connector for manual sync operations
// func GetConnector() *libsql.Connector {
// 	connectorMutex.RLock()
// 	defer connectorMutex.RUnlock()
// 	return globalConnector
// }

package db

import (
	"aether/internal/logging"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/glebarez/sqlite"
	"github.com/tursodatabase/go-libsql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

var (
	// Global connector for manual sync access
	globalConnector *libsql.Connector
	connectorMutex  sync.RWMutex
)

func Initialize() (*gorm.DB, error) {
	gormLogger := logger.Default.LogMode(logger.Info)

	libsqlURL := os.Getenv("LIBSQL_URL")
	if libsqlURL == "" {
		return nil, fmt.Errorf("LIBSQL_URL environment variable must be set")
	}

	authToken := os.Getenv("LIBSQL_AUTH_TOKEN")
	if authToken == "" {
		return nil, fmt.Errorf("LIBSQL_AUTH_TOKEN environment variable must be set")
	}

	log := logging.NewLogger()
	useReplica := os.Getenv("LIBSQL_USE_REPLICA") == "true"

	fmt.Println("useReplica ->", useReplica)

	var db *gorm.DB
	var err error

	if useReplica {
		log.Info("Using libSQL embedded replica mode", "syncUrl", libsqlURL)

		replicaPath := "./libsql-replica/local.db"

		if err = os.MkdirAll(filepath.Dir(replicaPath), 0755); err != nil {
			return nil, fmt.Errorf("failed to create replica dir: %w", err)
		}

		// Create connector with manual sync (no auto-sync interval)
		connector, err := libsql.NewEmbeddedReplicaConnector(
			replicaPath,
			libsqlURL,
			libsql.WithAuthToken(authToken),
			libsql.WithSyncInterval(0), // Disable auto-sync for better write performance
		)
		if err != nil {
			return nil, fmt.Errorf("failed to create embedded replica connector: %w", err)
		}

		// Store connector globally for manual sync access
		connectorMutex.Lock()
		globalConnector = connector
		connectorMutex.Unlock()

		sqlDB := sql.OpenDB(connector)

		// Pi-optimized connection pool settings
		sqlDB.SetMaxIdleConns(5)
		sqlDB.SetMaxOpenConns(10)
		sqlDB.SetConnMaxLifetime(time.Hour)
		sqlDB.SetConnMaxIdleTime(10 * time.Minute)

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
			Logger:      gormLogger,
			PrepareStmt: true,
		})
		if err != nil {
			connector.Close()
			return nil, fmt.Errorf("failed to open GORM with libSQL replica: %w", err)
		}

		// Apply Pi-optimized SQLite settings
		if err := applySQLiteOptimizations(db); err != nil {
			log.Warn("Failed to apply SQLite optimizations", "error", err)
		}

		// Start background sync
		syncInterval := getSyncInterval()
		startBackgroundSync(syncInterval)

		log.Info("Successfully connected to libSQL embedded replica")
		log.Info("Background sync enabled", "interval", syncInterval)

	} else {
		log.Info("Using libSQL HTTP connection", "url", libsqlURL)
		log.Warn("Direct HTTP mode may experience STREAM_EXPIRED errors. Consider setting LIBSQL_USE_REPLICA=true")

		sqlDB, err := sql.Open("libsql", libsqlURL)
		if err != nil {
			return nil, fmt.Errorf("failed to open libSQL connection: %w", err)
		}

		// Improved connection pool settings
		sqlDB.SetMaxIdleConns(2)
		sqlDB.SetMaxOpenConns(10)
		sqlDB.SetConnMaxLifetime(30 * time.Second)

		db, err = gorm.Open(sqlite.Dialector{
			Conn: sqlDB,
		}, &gorm.Config{
			Logger:      gormLogger,
			PrepareStmt: true,
		})
		if err != nil {
			return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
		}

		log.Info("Connected to libSQL HTTP mode with improved pooling")
	}

	return db, nil
}

// getSyncInterval returns sync interval from env or default (10 seconds)
func getSyncInterval() time.Duration {
	if interval := os.Getenv("LIBSQL_SYNC_INTERVAL"); interval != "" {
		if d, err := time.ParseDuration(interval); err == nil {
			return d
		}
	}
	return 10 * time.Second // Default: sync every 10 seconds
}

// startBackgroundSync starts a goroutine that periodically syncs with Turso
func startBackgroundSync(interval time.Duration) {
	go func() {
		log := logging.NewLogger()
		ticker := time.NewTicker(interval)
		defer ticker.Stop()

		for range ticker.C {
			connectorMutex.RLock()
			conn := globalConnector
			connectorMutex.RUnlock()

			if conn != nil {
				result, err := conn.Sync()
				if err != nil {
					log.Warn("Background sync failed", "error", err)
				} else if result.FramesSynced > 0 {
					log.Info("Background sync completed",
						"framesSynced", result.FramesSynced,
						"frameNo", result.FrameNo)
				}
			}
		}
	}()
}

// applySQLiteOptimizations applies Pi-optimized PRAGMA settings
// NOTE: We don't set WAL mode because libSQL manages its own replication WAL
func applySQLiteOptimizations(db *gorm.DB) error {
	log := logging.NewLogger()

	// Check current journal mode (for debugging)
	var journalMode string
	if err := db.Raw("PRAGMA journal_mode").Scan(&journalMode).Error; err == nil {
		log.Info("Current journal mode", "mode", journalMode)
	}

	// Pi-optimized pragmas (reduced memory for Raspberry Pi)
	pragmas := []string{
		// NOTE: Do NOT set "PRAGMA journal_mode = WAL"
		// libSQL embedded replica manages its own WAL for replication
		"PRAGMA synchronous = NORMAL",      // Balance safety and performance
		"PRAGMA cache_size = -32000",       // 32MB cache (Pi-friendly)
		"PRAGMA temp_store = MEMORY",       // Temp tables in RAM
		"PRAGMA mmap_size = 67108864",      // 64MB memory-mapped I/O (Pi-friendly)
		"PRAGMA page_size = 4096",          // Standard page size
		"PRAGMA busy_timeout = 10000",      // 10s timeout for Pi's slower I/O
		"PRAGMA foreign_keys = ON",         // Enable FK constraints
		"PRAGMA locking_mode = NORMAL",     // Allow multiple connections
		"PRAGMA auto_vacuum = INCREMENTAL", // Prevent bloat
	}

	for _, pragma := range pragmas {
		if err := db.Exec(pragma).Error; err != nil {
			// Log warnings but don't fail - some pragmas may not be supported
			log.Warn("Pragma execution warning", "pragma", pragma, "error", err)
		}
	}

	log.Info("SQLite optimizations applied (Pi-optimized)")
	return nil
}

func Migrate(db *gorm.DB) error {
	log := logging.NewLogger()

	log.Info("Running database migrations")

	// Temporarily disable foreign key constraints to allow schema changes
	if err := db.Exec("PRAGMA foreign_keys = OFF").Error; err != nil {
		log.Warn("Could not disable foreign keys", "error", err)
	}

	// AutoMigrate models
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

// GetConnector returns the global libSQL connector for manual sync operations
func GetConnector() *libsql.Connector {
	connectorMutex.RLock()
	defer connectorMutex.RUnlock()
	return globalConnector
}

// SyncNow performs an immediate sync with Turso (useful for manual sync endpoint)
func SyncNow() (frames int, err error) {
	connectorMutex.RLock()
	conn := globalConnector
	connectorMutex.RUnlock()

	if conn == nil {
		return 0, fmt.Errorf("no connector available (not in replica mode)")
	}

	result, err := conn.Sync()
	if err != nil {
		return 0, err
	}

	return result.FramesSynced, nil
}
