# Create Migration Command

## Purpose
This command generates a new database migration file for the Aether backend. Only use it for changes that automigrate can't handle effectively 

## Usage
User can request a migration by saying:
- "Create a migration to [description]"
- "Generate a migration called [name]"
- "Make a migration for [purpose]"

## Implementation

When user requests a migration, create a file following these specifications:

### File Location
```
backend/internal/db/migrations/
```

### File Naming
```
YYYYMMDD_HHMMSS_description.go
```
- Use current timestamp (year, month, day, hour, minute, second)
- Convert description to snake_case
- Example: `20260107_153422_add_default_tags.go`

### File Template

```go
package migrations

import (
    "aether/internal/logging"
    "gorm.io/gorm"
)

func init() {
    Register(Migration{
        Version: "YYYYMMDD_HHMMSS_description",
        Name:    "Human readable description",
        NoTransaction: false, // Set to true if you need to change PRAGMA settings
        Up: func(tx *gorm.DB) error {
            log := logging.NewLogger()
            log.Info("Running migration: description")
            
            // TODO: Add migration logic here
            
            return nil
        },
        Down: func(tx *gorm.DB) error {
            log := logging.NewLogger()
            log.Info("Rolling back migration: description")
            
            // TODO: Add rollback logic here
            // Return nil if rollback not supported
            
            return nil
        },
    })
}
```

### Rules

1. **Version field** must match the filename timestamp
2. **Name field** should be a human-readable description
3. **Up function** contains the migration logic
4. **Down function** contains the rollback logic (or explanation why not possible)
5. Always use logging for visibility
6. Operations run in a transaction automatically
7. Return errors appropriately
8. Add TODO comments for user to fill in logic

### Examples

**User Request**: "Create a migration to add default user preferences"

**Generated File**: `backend/internal/db/migrations/20260107_153422_add_default_user_preferences.go`

```go
package migrations

import (
    "aether/internal/logging"
    "gorm.io/gorm"
)

func init() {
    Register(Migration{
        Version: "20260107_153422_add_default_user_preferences",
        Name:    "Add default user preferences",
        Up: func(tx *gorm.DB) error {
            log := logging.NewLogger()
            log.Info("Running migration: Add default user preferences")
            
            // TODO: Add logic to create default user preferences
            
            return nil
        },
        Down: func(tx *gorm.DB) error {
            log := logging.NewLogger()
            log.Info("Rolling back migration: Add default user preferences")
            
            // TODO: Add rollback logic
            
            return nil
        },
    })
}
```

### After Creation

1. Inform user the migration file was created
2. Show the file path
3. Remind user to implement the TODO sections
4. Note that migration will run automatically on next app startup

