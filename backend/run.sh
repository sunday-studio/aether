#!/bin/bash
# Run script for Aether backend with libSQL support
# Note: macOS doesn't support allowing duplicate symbols via linker flags
# The duplicate symbol error must be resolved by avoiding the conflict

set -e

# Detect OS
OS="$(uname -s)"

# Set minimum macOS version to suppress warnings (optional)
if [[ "$OS" == "Darwin" ]]; then
    export CGO_CFLAGS="-mmacosx-version-min=15.5"
    export CGO_LDFLAGS="-mmacosx-version-min=15.5"
fi

echo "🚀 Running Aether backend..."
echo "   OS: $OS"
echo ""
echo "⚠️  Note: If you encounter duplicate symbol errors, you may need to:"
echo "   1. Use only libSQL (set LIBSQL_URL) and avoid importing gorm.io/driver/sqlite"
echo "   2. Or use only local SQLite (unset LIBSQL_URL)"
echo ""

# Run the application
go run ./main.go

