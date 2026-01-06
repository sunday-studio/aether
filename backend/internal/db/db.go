package db

import (
	"aether/internal/logging"
	"database/sql"
	"fmt"
	"net/url"
	"os"
	"path/filepath"

	_ "github.com/tursodatabase/go-libsql"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

func Initialize() (*gorm.DB, error) {
	gormLogger := logger.Default.LogMode(logger.Info)

	// Check if libSQL URL is provided, otherwise fall back to SQLite
	libsqlURL := os.Getenv("LIBSQL_URL")
	libsqlAuthToken := os.Getenv("LIBSQL_AUTH_TOKEN")

	// Note: Using libSQL with gorm.io/driver/sqlite causes duplicate symbol errors on macOS
	// because both include SQLite C code. Only use libSQL when explicitly set via LIBSQL_URL.

	var db *gorm.DB
	var err error

	if libsqlURL != "" {
		// Use libSQL server (local Docker instance or remote)
		log := logging.NewLogger()
		log.Info("Connecting to libSQL server", "url", libsqlURL)

		// Build the connection URL with auth token if provided
		// The libSQL driver supports http://, https://, and libsql:// URLs
		connURL := libsqlURL
		if libsqlAuthToken != "" {
			u, err := url.Parse(libsqlURL)
			if err != nil {
				return nil, fmt.Errorf("invalid libSQL URL: %w", err)
			}
			q := u.Query()
			q.Set("authToken", libsqlAuthToken)
			u.RawQuery = q.Encode()
			connURL = u.String()
		}

		// Use the libSQL driver directly - it automatically handles remote connections
		sqlDB, err := sql.Open("libsql", connURL)
		if err != nil {
			return nil, fmt.Errorf("failed to open libSQL connection: %w", err)
		}

		// Use GORM with the libSQL connection
		// libSQL is SQLite-compatible, so we can use the SQLite dialector
		// Note: This may cause duplicate symbol warnings at link time, but should still work
		db, err = gorm.Open(sqlite.Dialector{Conn: sqlDB}, &gorm.Config{
			Logger: gormLogger,
		})
		if err != nil {
			return nil, fmt.Errorf("failed to open GORM with libSQL: %w", err)
		}
	} else {
		// Fall back to local SQLite for development
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
	}

	sqlDB, err := db.DB()
	if err != nil {
		return nil, err
	}

	sqlDB.SetMaxIdleConns(10)
	sqlDB.SetMaxOpenConns(100)

	return db, nil
}

func Migrate(db *gorm.DB) error {
	log := logging.NewLogger()

	log.Info("Running database migrations")

	return db.AutoMigrate(&Entry{}, &Tag{}, &Task{}, &Goal{}, &GoalInstance{})
}
