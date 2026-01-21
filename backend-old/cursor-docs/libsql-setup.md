# libSQL Server Setup

This document describes how to set up a self-hosted libSQL server for the Aether backend.

## Quick Start with Docker

The easiest way to run libSQL server is using Docker Compose:

```bash
docker-compose -f docker-compose.libsql.yml up -d
```

This will start the libSQL server on:
- HTTP endpoint: `http://localhost:8080`
- GRPC endpoint: `localhost:5001`
- Data directory: `./libsql-data`

## Manual Setup

### Option 1: Using Docker

```bash
docker run -d \
  --name libsql-server \
  -p 8080:8080 \
  -p 5001:5001 \
  -v $(pwd)/libsql-data:/data \
  ghcr.io/tursodatabase/libsql-server:latest \
  --grpc-listen-addr 0.0.0.0:5001 \
  --http-listen-addr 0.0.0.0:8080 \
  --data-dir /data
```

### Option 2: Build from Source

```bash
git clone https://github.com/tursodatabase/libsql.git
cd libsql
cargo build --release
./target/release/sqld --grpc-listen-addr 0.0.0.0:5001 --http-listen-addr 0.0.0.0:8080
```

## Configuration

Set the following environment variables in your backend:

```bash
LIBSQL_URL=http://your-server:8080
LIBSQL_AUTH_TOKEN=your-auth-token  # Optional, if authentication is enabled
```

For local development:
```bash
LIBSQL_URL=http://localhost:8080
```

If `LIBSQL_URL` is not set, the backend will fall back to using local SQLite.

## Production Deployment

For production, consider:
- Setting up authentication (JWT tokens)
- Using HTTPS/WSS instead of HTTP/WS
- Configuring firewall rules to restrict access
- Setting up proper backup procedures
- Using a reverse proxy (nginx, traefik, etc.)

## Data Migration

See `migrate-to-libsql.sh` for instructions on migrating data from SQLite to libSQL.

