package db

import (
	"aether/internal/logging"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"

	"github.com/tursodatabase/go-libsql"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

func Initialize() (*gorm.DB, error) {
	gormLogger := logger.Default.LogMode(logger.Info)

	// Check if libSQL URL is provided, otherwise fall back to SQLite
	libsqlURL := os.Getenv("LIBSQL_URL")
	libsqlAuthToken := os.Getenv("LIBSQL_AUTH_TOKEN")

	var db *gorm.DB
	var err error

	if libsqlURL != "" {
		// Use libSQL server
		log := logging.NewLogger()
		log.Info("Connecting to libSQL server", "url", libsqlURL)

		// Create libSQL connector for remote server
		// go-libsql provides connectors for remote libSQL servers
		opts := []libsql.ConnectorOption{}
		if libsqlAuthToken != "" {
			opts = append(opts, libsql.WithAuthToken(libsqlAuthToken))
		}

		// Create remote connector for direct connection to libSQL server
		connector, err := libsql.NewRemoteConnector(libsqlURL, opts...)
		if err != nil {
			return nil, fmt.Errorf("failed to create libSQL connector: %w", err)
		}

		// Open database/sql connection using the connector
		sqlDB := sql.OpenDB(connector)

		// Use GORM with the libSQL connection via sqlite driver
		// libSQL is SQLite-compatible, so we can use sqlite driver
		// The sqlite.Dialector accepts a *sql.DB connection
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

	fmt.Println("APP_ENV", os.Getenv("APP_ENV"))

	if err := db.AutoMigrate(&Entry{}, &Tag{}, &Task{}, &Goal{}, &GoalInstance{}); err != nil {
		log.Error("Migration failed", "error", err)
		return err
	}

	// if err := SeedDatabase(db); err != nil {
	// 	log.Error("Seeding database failed", "error", err)
	// 	return err
	// }

	log.Info("Database migrations completed successfully")
	return nil
}

// 	if os.Getenv("APP_ENV") == "development" {
// 	db.SeedDatabase(db)
// }
