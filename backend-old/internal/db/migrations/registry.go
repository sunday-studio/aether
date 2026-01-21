package migrations

import (
	"sort"

	"gorm.io/gorm"
)

// Migration represents a database migration
type Migration struct {
	Version       string
	Name          string
	Up            func(tx *gorm.DB) error
	Down          func(tx *gorm.DB) error
	NoTransaction bool // Set to true to run without transaction wrapping (needed for PRAGMA changes)
}

var registry []Migration

// Register adds a migration to the registry
func Register(m Migration) {
	registry = append(registry, m)
}

// GetAll returns all registered migrations sorted by version
func GetAll() []Migration {
	// Create a copy to avoid external modifications
	migrations := make([]Migration, len(registry))
	copy(migrations, registry)

	// Sort by version (timestamp-based)
	sort.Slice(migrations, func(i, j int) bool {
		return migrations[i].Version < migrations[j].Version
	})

	return migrations
}

// GetByVersion finds a migration by its version
func GetByVersion(version string) *Migration {
	for _, m := range registry {
		if m.Version == version {
			return &m
		}
	}
	return nil
}
