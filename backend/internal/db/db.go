package db

import (
	"aether/internal/logging"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/glebarez/sqlite"
	"github.com/tursodatabase/go-libsql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

var (
	globalConnector   *libsql.Connector
	connectorMutex    sync.RWMutex
	hasSyncCapability bool
)

func Initialize() (*gorm.DB, error) {
	gormLogger := logger.Default.LogMode(logger.Info)
	log := logging.NewLogger()

	libsqlURL := cleanEnvVar(os.Getenv("LIBSQL_URL"))
	authToken := cleanEnvVar(os.Getenv("LIBSQL_AUTH_TOKEN"))

	replicaPath := "./libsql-replica/local.db"
	replicaDir := filepath.Dir(replicaPath)

	if err := os.MkdirAll(replicaDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create replica dir: %w", err)
	}

	var db *gorm.DB
	var err error
	hasRemote := libsqlURL != ""

	if !hasRemote {
		if hasReplicaMetadata(replicaPath) {
			if err := cleanupReplicaMetadata(replicaPath, log); err != nil {
				log.Warn("Failed to clean replica metadata", "error", err)
			}
		}

		sqlDB, err := sql.Open("libsql", fmt.Sprintf("file:%s", replicaPath))
		if err != nil {
			return nil, fmt.Errorf("failed to open local libSQL database: %w", err)
		}

		sqlDB.SetMaxIdleConns(5)
		sqlDB.SetMaxOpenConns(10)
		sqlDB.SetConnMaxLifetime(time.Hour)
		sqlDB.SetConnMaxIdleTime(10 * time.Minute)

		db, err = gorm.Open(sqlite.Dialector{
			Conn: sqlDB,
		}, &gorm.Config{
			Logger:      gormLogger,
			PrepareStmt: true,
		})
		if err != nil {
			sqlDB.Close()
			return nil, fmt.Errorf("failed to open GORM with local libSQL: %w", err)
		}

		connectorMutex.Lock()
		globalConnector = nil
		hasSyncCapability = false
		connectorMutex.Unlock()

	} else {
		if err := checkAndCleanCorruptedReplica(replicaPath, log); err != nil {
			log.Warn("Failed to check/clean replica", "error", err)
		}

		connectorOpts := []libsql.Option{
			libsql.WithSyncInterval(0),
		}

		if authToken != "" {
			connectorOpts = append(connectorOpts, libsql.WithAuthToken(authToken))
		}

		var connector *libsql.Connector
		connector, err = libsql.NewSyncedDatabaseConnector(
			replicaPath,
			libsqlURL,
			connectorOpts...,
		)

		if err != nil {
			connector, err = libsql.NewEmbeddedReplicaConnector(
				replicaPath,
				libsqlURL,
				connectorOpts...,
			)

			if err != nil {
				if cleanErr := cleanupCorruptedReplica(replicaPath, log); cleanErr != nil {
					return nil, fmt.Errorf("failed to cleanup replica: %w", cleanErr)
				}

				connector, err = libsql.NewEmbeddedReplicaConnector(
					replicaPath,
					libsqlURL,
					connectorOpts...,
				)
				if err != nil {
					return nil, fmt.Errorf("failed to create synced connector after cleanup: %w", err)
				}
			}
		}

		connectorMutex.Lock()
		globalConnector = connector
		hasSyncCapability = true
		connectorMutex.Unlock()

		sqlDB := sql.OpenDB(connector)

		sqlDB.SetMaxIdleConns(5)
		sqlDB.SetMaxOpenConns(10)
		sqlDB.SetConnMaxLifetime(time.Hour)
		sqlDB.SetConnMaxIdleTime(10 * time.Minute)

		connector.Sync()

		db, err = gorm.Open(sqlite.Dialector{
			Conn: sqlDB,
		}, &gorm.Config{
			Logger:      gormLogger,
			PrepareStmt: true,
		})
		if err != nil {
			connector.Close()
			return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
		}

		startBackgroundSync(getSyncInterval())
	}

	if err := applySQLiteOptimizations(db); err != nil {
		log.Warn("Failed to apply SQLite optimizations", "error", err)
	}

	return db, nil
}

func hasReplicaMetadata(replicaPath string) bool {
	metadataPatterns := []string{
		replicaPath + "-shm",
		replicaPath + "-wal",
		filepath.Join(filepath.Dir(replicaPath), ".meta"),
	}

	for _, pattern := range metadataPatterns {
		if _, err := os.Stat(pattern); err == nil {
			return true
		}
	}
	return false
}

func cleanupReplicaMetadata(replicaPath string, log *logging.Logger) error {
	replicaDir := filepath.Dir(replicaPath)
	metadataFiles := []string{
		replicaPath + "-shm",
		replicaPath + "-wal",
		filepath.Join(replicaDir, ".meta"),
	}

	for _, file := range metadataFiles {
		if err := os.Remove(file); err != nil && !os.IsNotExist(err) {
			log.Warn("Failed to remove metadata file", "file", file, "error", err)
		}
	}

	return nil
}

func cleanEnvVar(value string) string {
	value = strings.TrimSpace(value)
	value = strings.Trim(value, `"'`)
	value = strings.TrimSpace(value)
	return value
}

func checkAndCleanCorruptedReplica(replicaPath string, log *logging.Logger) error {
	if _, err := os.Stat(replicaPath); err != nil {
		return nil
	}

	metadataPatterns := []string{
		replicaPath + "-shm",
		replicaPath + "-wal",
		filepath.Join(filepath.Dir(replicaPath), ".meta"),
	}

	hasMetadata := false
	for _, pattern := range metadataPatterns {
		if _, err := os.Stat(pattern); err == nil {
			hasMetadata = true
			break
		}
	}

	if !hasMetadata {
		return cleanupCorruptedReplica(replicaPath, log)
	}

	return nil
}

func cleanupCorruptedReplica(replicaPath string, log *logging.Logger) error {
	replicaDir := filepath.Dir(replicaPath)

	if err := os.RemoveAll(replicaDir); err != nil {
		return fmt.Errorf("failed to remove replica directory: %w", err)
	}

	if err := os.MkdirAll(replicaDir, 0755); err != nil {
		return fmt.Errorf("failed to recreate replica directory: %w", err)
	}

	return nil
}

func getSyncInterval() time.Duration {
	interval := cleanEnvVar(os.Getenv("LIBSQL_SYNC_INTERVAL"))
	if interval != "" {
		if d, err := time.ParseDuration(interval + "s"); err == nil {
			return d
		}
		if d, err := time.ParseDuration(interval); err == nil {
			return d
		}
	}
	return 10 * time.Second
}

func startBackgroundSync(interval time.Duration) {
	go func() {
		ticker := time.NewTicker(interval)
		defer ticker.Stop()

		for range ticker.C {
			connectorMutex.RLock()
			conn := globalConnector
			connectorMutex.RUnlock()

			if conn != nil {
				conn.Sync()
			}
		}
	}()
}

func applySQLiteOptimizations(db *gorm.DB) error {
	pragmas := []string{
		"PRAGMA synchronous = NORMAL",
		"PRAGMA cache_size = -32000",
		"PRAGMA temp_store = MEMORY",
		"PRAGMA mmap_size = 67108864",
		"PRAGMA page_size = 4096",
		"PRAGMA busy_timeout = 10000",
		"PRAGMA foreign_keys = ON",
		"PRAGMA locking_mode = NORMAL",
		"PRAGMA auto_vacuum = INCREMENTAL",
	}

	for _, pragma := range pragmas {
		db.Exec(pragma)
	}

	return nil
}

func Migrate(db *gorm.DB) error {
	log := logging.NewLogger()

	db.Exec("PRAGMA foreign_keys = OFF")

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
			db.Exec("PRAGMA foreign_keys = ON")
			log.Error("Failed to migrate model", "error", err, "model", fmt.Sprintf("%T", model))
			return fmt.Errorf("failed to migrate %T: %w", model, err)
		}
	}

	db.Exec("PRAGMA foreign_keys = ON")
	return nil
}

func GetConnector() *libsql.Connector {
	connectorMutex.RLock()
	defer connectorMutex.RUnlock()
	return globalConnector
}

func SyncNow() (frames int, err error) {
	connectorMutex.RLock()
	conn := globalConnector
	canSync := hasSyncCapability
	connectorMutex.RUnlock()

	if !canSync {
		return 0, fmt.Errorf("sync not available: no LIBSQL_URL configured - set LIBSQL_URL to enable sync")
	}

	if conn == nil {
		return 0, fmt.Errorf("no connector available")
	}

	result, err := conn.Sync()
	if err != nil {
		return 0, err
	}

	return result.FramesSynced, nil
}

func HasSyncCapability() bool {
	connectorMutex.RLock()
	defer connectorMutex.RUnlock()
	return hasSyncCapability
}
