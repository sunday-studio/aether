# Database Migrations

This directory contains data migrations for the Aether backend. Schema migrations are handled by GORM's AutoMigrate, while this system tracks and executes data transformations and fixes.

## Overview

The migration system:
- Runs automatically on application startup
- Tracks applied migrations in the `schema_migrations` table
- Executes migrations in transactions for safety
- Supports rollback functionality
- Works safely with libSQL replica sync

## Creating a New Migration

### Using Cursor AI (Recommended)

Simply ask Cursor AI to create a migration using natural language:

```
"Create a migration to add default user settings"
"Generate a migration called fix_task_timestamps"
"Create a migration to populate tag colors"
```

Cursor will create the properly formatted migration file for you.

### Naming Convention

Migration files follow this format:
```
YYYYMMDD_HHMMSS_description.go
```

Examples:
- `20260107_153000_fix_corrupted_ids.go`
- `20260108_101500_add_default_tags.go`
- `20260115_094530_populate_user_preferences.go`

**Important**: The timestamp ensures migrations run in order and prevents conflicts across branches.

## Migration Template

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
        Up: func(tx *gorm.DB) error {
            log := logging.NewLogger()
            log.Info("Running migration: description")
            
            // Your migration logic here
            // This runs in a transaction
            
            return nil
        },
        Down: func(tx *gorm.DB) error {
            log := logging.NewLogger()
            log.Info("Rolling back migration: description")
            
            // Your rollback logic here
            // Return nil if rollback not supported
            
            return nil
        },
    })
}
```

## Migration Guidelines

### Do's

- Keep migrations focused on a single task
- Use transactions (automatically provided by the runner)
- Add logging for visibility
- Handle errors gracefully
- Test migrations on a copy of production data
- Consider rollback implications

### Don'ts

- Don't modify existing migration files
- Don't use the same version/timestamp for multiple migrations
- Don't perform operations that can't be rolled back without noting it
- Don't rely on application models (they may change over time)

## Running Migrations

Migrations run automatically when the application starts:

```bash
go run main.go
```

The output will show:
```
Running migration: Fix corrupted IDs
Migration completed: 20260107_153000_fix_corrupted_ids
All migrations completed: 1
```

## Rollback Migrations

To rollback migrations programmatically:

```go
import "aether/internal/db/migrations"

// Rollback the last migration
err := migrations.Rollback(db, 1)

// Rollback the last 3 migrations
err := migrations.Rollback(db, 3)
```

## Migration Status

To check migration status:

```go
import "aether/internal/db/migrations"

statuses, err := migrations.GetStatus(db)
for _, status := range statuses {
    if status.Applied {
        fmt.Printf("[APPLIED] %s - %s (applied: %s)\n", 
            status.Version, status.Name, status.AppliedAt)
    } else {
        fmt.Printf("[PENDING] %s - %s\n", 
            status.Version, status.Name)
    }
}
```

## Common Migration Patterns

### Data Transformation

```go
Up: func(tx *gorm.DB) error {
    // Update records in batches
    var tasks []db.Task
    if err := tx.Find(&tasks).Error; err != nil {
        return err
    }
    
    for _, task := range tasks {
        // Transform data
        task.SomeField = transformValue(task.SomeField)
        if err := tx.Save(&task).Error; err != nil {
            return err
        }
    }
    
    return nil
}
```

### Populating Default Data

```go
Up: func(tx *gorm.DB) error {
    defaults := []db.Tag{
        {Name: "Important"},
        {Name: "Work"},
        {Name: "Personal"},
    }
    
    for _, tag := range defaults {
        if err := tx.Create(&tag).Error; err != nil {
            return err
        }
    }
    
    return nil
}
```

### Fixing Data Issues

```go
Up: func(tx *gorm.DB) error {
    // Find and fix invalid records
    result := tx.Exec(`
        UPDATE tasks 
        SET due_date = NULL 
        WHERE due_date < created_at
    `)
    
    return result.Error
}
```

## LibSQL Sync Considerations

- Migrations run **after** initial libSQL sync completes
- Migration changes automatically sync to remote libSQL
- Each migration runs in its own transaction
- The `schema_migrations` table syncs across all replicas
- Concurrent instances are protected by database locks

## Troubleshooting

### Migration fails to run

1. Check the logs for specific error messages
2. Verify database connectivity
3. Ensure no syntax errors in the migration
4. Test the migration logic separately

### Migration already applied

The system tracks applied migrations in `schema_migrations`. If a migration shows as applied but you need to re-run it:

1. Remove the entry from `schema_migrations` table
2. Restart the application

### Need to skip a migration

Remove or comment out the `Register()` call in the migration file's `init()` function.

## File Structure

```
backend/internal/db/migrations/
├── registry.go                            # Migration registry
├── runner.go                              # Migration execution engine
├── 20260107_153000_fix_corrupted_ids.go  # Example migration
└── ...                                    # Your migrations
```

## Best Practices

1. **Version Control**: Commit migration files with the code that requires them
2. **Testing**: Test migrations on a copy of production data before deploying
3. **Documentation**: Add comments explaining what the migration does and why
4. **Idempotency**: Where possible, make migrations idempotent
5. **Performance**: For large datasets, consider batching operations
6. **Rollback**: Always implement Down() or explain why it's not possible

## Examples

See `20260107_153000_fix_corrupted_ids.go` for a complete example migration that:
- Checks multiple tables
- Logs progress
- Handles errors
- Explains rollback limitations

