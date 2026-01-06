#!/bin/bash
set -e

# Script to migrate data from SQLite to libSQL
# Usage: ./migrate-to-libsql.sh [libsql-url] [auth-token]

LIBSQL_URL=${1:-"http://localhost:8080"}
AUTH_TOKEN=${2:-""}

echo "🔄 Migrating data from SQLite to libSQL..."
echo "Target: $LIBSQL_URL"

# Export SQLite data
echo "📤 Exporting SQLite data..."
SQLITE_DB="sqlite/aether.db"
BACKUP_FILE="aether_backup_$(date +%Y%m%d_%H%M%S).sql"

if [ ! -f "$SQLITE_DB" ]; then
    echo "❌ SQLite database not found at $SQLITE_DB"
    exit 1
fi

sqlite3 "$SQLITE_DB" .dump > "$BACKUP_FILE"
echo "✅ Exported to $BACKUP_FILE"

# Import to libSQL
echo "📥 Importing to libSQL..."

# Note: You'll need to use libSQL client or HTTP API to import
# This is a placeholder - actual implementation depends on libSQL client tools
echo "⚠️  Manual step required:"
echo "   1. Use libSQL client to connect to $LIBSQL_URL"
echo "   2. Execute the SQL from $BACKUP_FILE"
echo "   3. Or use libSQL HTTP API to import the data"

# Example using curl (if libSQL HTTP API supports it):
# curl -X POST "$LIBSQL_URL/v1/execute" \
#   -H "Content-Type: application/json" \
#   -H "Authorization: Bearer $AUTH_TOKEN" \
#   -d @- << EOF
# {
#   "sql": "$(cat $BACKUP_FILE | tr '\n' ' ')"
# }
# EOF

echo "✅ Migration script completed"
echo "📝 Backup saved to: $BACKUP_FILE"

