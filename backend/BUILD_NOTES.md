# Build Notes for libSQL Integration

## Known Issues

### 1. Duplicate Symbol Error (macOS)

**Critical**: On macOS, you **cannot** use both `gorm.io/driver/sqlite` and `github.com/tursodatabase/go-libsql` together because:

1. Both libraries include SQLite C code
2. macOS linker doesn't support allowing duplicate symbols (unlike Linux)
3. This causes build failures with duplicate symbol errors

**Current Behavior:**
- By default, the app uses local SQLite (no conflict)
- libSQL is only used when `LIBSQL_URL` is explicitly set
- When `LIBSQL_URL` is set, you'll encounter the duplicate symbol error

**Workaround Options:**

1. **Use only local SQLite** (default): Don't set `LIBSQL_URL`
2. **Use only libSQL**: Remove `gorm.io/driver/sqlite` from imports and use libSQL exclusively
3. **Use build tags**: Create separate builds for libSQL vs SQLite (complex)

### 2. macOS Version Warnings

You may see warnings about object files being built for macOS 15.5 while linking against 15.0. These are harmless but can be suppressed.

## Solution

### Quick Run (Recommended for Development)

Use the provided run script:

```bash
./run.sh
```

This script automatically handles:
- Duplicate symbol resolution with appropriate linker flags
- macOS version warning suppression
- Cross-platform compatibility

### Quick Build

Use the provided build script:

```bash
./build.sh
```

This script automatically handles:
- Duplicate symbol resolution with appropriate linker flags
- macOS version warning suppression
- Cross-platform compatibility

### Manual Run/Build

#### macOS

For running:
```bash
export CGO_LDFLAGS="-Wl,-allow_duplicate_symbols -mmacosx-version-min=15.5"
export CGO_CFLAGS="-mmacosx-version-min=15.5"
go run -ldflags="-Wl,-allow_duplicate_symbols" ./main.go
```

For building:
```bash
export CGO_LDFLAGS="-Wl,-allow_duplicate_symbols -mmacosx-version-min=15.5"
export CGO_CFLAGS="-mmacosx-version-min=15.5"
go build -ldflags="-Wl,-allow_duplicate_symbols" -o aether-backend ./main.go
```

#### Linux

For running:
```bash
export CGO_LDFLAGS="-Wl,--allow-multiple-definition"
go run -ldflags="-Wl,--allow-multiple-definition" ./main.go
```

For building:
```bash
export CGO_LDFLAGS="-Wl,--allow-multiple-definition"
go build -ldflags="-Wl,--allow-multiple-definition" -o aether-backend ./main.go
```

### Docker Build

If using Docker, update your Dockerfile to include the linker flags in the build step.

## Current Status

The code is configured to:
- Use libSQL when `LIBSQL_URL` is set (defaults to `http://localhost:8080`)
- Fall back to local SQLite when `LIBSQL_URL` is empty

The duplicate symbol issue only affects the build process, not runtime functionality, once resolved with linker flags.

