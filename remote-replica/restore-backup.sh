#!/bin/bash

# LibSQL Database Restore Script
# Run this on your Mac Mini to restore from a backup

set -e

echo "🔄 LibSQL Database Restore"
echo "=========================="
echo ""

# Check if backups exist
if [ ! -d "libsql-backups" ]; then
    echo "❌ Error: libsql-backups directory not found"
    exit 1
fi

# List available backups
echo "📦 Available backups:"
echo ""
BACKUPS=($(ls -1t libsql-backups/backup-*.sqld.gz 2>/dev/null))

if [ ${#BACKUPS[@]} -eq 0 ]; then
    echo "❌ No backups found in libsql-backups/"
    exit 1
fi

# Display backups with numbers
for i in "${!BACKUPS[@]}"; do
    BACKUP_FILE="${BACKUPS[$i]}"
    BACKUP_NAME=$(basename "$BACKUP_FILE")
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    BACKUP_DATE=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "$BACKUP_FILE" 2>/dev/null || stat -c "%y" "$BACKUP_FILE" 2>/dev/null | cut -d'.' -f1)
    echo "  [$((i+1))] $BACKUP_NAME"
    echo "      Size: $BACKUP_SIZE | Modified: $BACKUP_DATE"
    echo ""
done

# Prompt for selection
echo "Select a backup to restore (1-${#BACKUPS[@]}):"
read -p "Enter number: " BACKUP_NUM

# Validate input
if ! [[ "$BACKUP_NUM" =~ ^[0-9]+$ ]] || [ "$BACKUP_NUM" -lt 1 ] || [ "$BACKUP_NUM" -gt ${#BACKUPS[@]} ]; then
    echo "❌ Invalid selection"
    exit 1
fi

SELECTED_BACKUP="${BACKUPS[$((BACKUP_NUM-1))]}"
echo ""
echo "Selected backup: $(basename "$SELECTED_BACKUP")"
echo ""

# Check if server is running
echo "🔍 Checking libSQL server status..."
if docker ps | grep -q libsql-server; then
    echo "⚠️  LibSQL server is currently running"
    read -p "Stop the server to perform restore? (y/N): " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "Restore cancelled."
        exit 0
    fi
    echo "Stopping libSQL server..."
    docker-compose stop libsql-server
    echo "✓ Server stopped"
fi

echo ""
echo "⚠️  WARNING: This will replace the current database!"
echo "   Current database will be backed up to: libsql-data/data.sqld.pre-restore"
read -p "Are you sure you want to continue? (y/N): " final_confirm

if [ "$final_confirm" != "y" ] && [ "$final_confirm" != "Y" ]; then
    echo "Restore cancelled."
    echo "Restarting libSQL server..."
    docker-compose up -d libsql-server
    exit 0
fi

echo ""
echo "🔄 Starting restore process..."

# Backup current database
if [ -f "libsql-data/data.sqld" ]; then
    echo "📦 Backing up current database..."
    cp libsql-data/data.sqld "libsql-data/data.sqld.pre-restore-$(date +%Y%m%d-%H%M%S)"
    echo "✓ Current database backed up"
fi

# Decompress and restore
echo "📥 Decompressing backup..."
TEMP_FILE=$(mktemp)
gunzip -c "$SELECTED_BACKUP" > "$TEMP_FILE"
echo "✓ Backup decompressed"

echo "📝 Restoring database..."
cp "$TEMP_FILE" libsql-data/data.sqld
rm "$TEMP_FILE"
echo "✓ Database restored"

# Restart server
echo "🚀 Starting libSQL server..."
docker-compose up -d libsql-server

# Wait for health check
echo "⏳ Waiting for server to be healthy..."
sleep 3
for i in {1..10}; do
    if docker exec libsql-server curl -sf http://localhost:8080/health > /dev/null 2>&1; then
        echo "✓ Server is healthy"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "⚠️  Server health check timeout. Check logs: docker-compose logs libsql-server"
        exit 1
    fi
    sleep 2
done

echo ""
echo "✅ Restore completed successfully!"
echo ""
echo "📊 Post-restore checklist:"
echo "  1. Verify data: Connect from your Pi and check data"
echo "  2. Check logs: docker-compose logs -f libsql-server"
echo "  3. Test writes: Ensure writes are working"
echo ""
echo "💡 If something went wrong, you can restore the pre-restore backup:"
echo "   ls -lh libsql-data/data.sqld.pre-restore-*"
echo ""