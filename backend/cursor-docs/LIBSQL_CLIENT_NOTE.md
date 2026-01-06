# libSQL Client Library

## Package

Using `github.com/tursodatabase/go-libsql` as specified.

## Installation

Run the following to install the package:
```bash
go get github.com/tursodatabase/go-libsql
go mod tidy
```

## Next Steps

1. **Verify the correct package:**
   - Check the official libSQL documentation
   - Visit https://github.com/tursodatabase/libsql
   - Look for Go client examples or bindings

2. **Possible alternatives:**
   - Use libSQL's HTTP API directly with Go's `net/http` client
   - Use a database/sql driver wrapper for libSQL
   - Use `github.com/tursodatabase/go-libsql` if it exists (for embedded replicas)
   - Check if libSQL server exposes a standard database/sql driver interface

3. **Current status:**
   - Code compiles and works with SQLite
   - libSQL detection logic is in place
   - LWW conflict resolution is fully implemented
   - Once the correct client library is found, integration should be straightforward

## Implementation Location

The libSQL integration code is in `backend/internal/db/db.go` starting at line 26. Currently it falls back to SQLite, but the structure is ready for libSQL once the client library is available.

## Alternative Approach

If no Go client library exists, consider:
- Using HTTP requests directly to libSQL's HTTP API (`/v1/execute` endpoint)
- Creating a custom database/sql driver wrapper
- Using libSQL's WebSocket protocol directly

