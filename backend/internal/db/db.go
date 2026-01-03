package db

import (
	"aether/internal/logging"
	"fmt"
	"os"
	"path/filepath"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

func Initialize() (*gorm.DB, error) {
	dataDir := "sqlite"
	if err := os.MkdirAll(dataDir, 0755); err != nil {
		return nil, err
	}

	dbPath := filepath.Join(dataDir, "aether.db")

	gormLogger := logger.Default.LogMode(logger.Info)

	db, err := gorm.Open(sqlite.Open(dbPath), &gorm.Config{
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

	return db, nil
}

func Migrate(db *gorm.DB) error {
	log := logging.NewLogger()

	log.Info("Running database migrations")

	fmt.Println("APP_ENV", os.Getenv("APP_ENV"))

	if err := db.AutoMigrate(&Entry{}, &Tag{}, &Task{}, &SubTask{}, &Goal{}, &GoalInstance{}); err != nil {
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
