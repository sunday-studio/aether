#!/bin/bash
# Run script for Aether backend with libSQL

set -e

echo "🚀 Running Aether backend with libSQL (pure Go)..."
echo "   LIBSQL_URL: ${LIBSQL_URL:-not set}"
echo ""

# No special CGO flags needed for glebarez/sqlite
go run ./main.go
