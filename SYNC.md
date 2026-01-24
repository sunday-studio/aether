# Aether Sync

Offline-first, end-to-end encrypted sync. Your data is encrypted on the client; the server only stores opaque blobs.

## Architecture

- **Client**: SQLite + triggers → `_sync_outbox`. Sync engine encrypts (ChaCha20-Poly1305), pushes/pulls via HTTP.
- **Server**: Stores encrypted changes and blobs. No decryption.
- **Conflict resolution**: Last-write-wins by `_updated_at`.

## Setup

1. **Deploy the sync server** (Docker):

   ```bash
   cd sync-server && docker compose up -d
   ```

2. **In Aether**: Settings → Sync. Enter:
   - **Server URL**: `http://your-host:8080`
   - **Passphrase**: min 12 characters (used to derive the encryption key).

3. **Save** to configure. **Sync now** to run immediately.

## Security

- Key derived from passphrase + salt (Argon2id). Salt and `key_check` hash stored in `_sync_meta`.
- Passphrase is not sent to the server. It is kept in memory on the client only.
- Use HTTPS in production (put the server behind TLS).

## Development

- `sync-server/`: Axum server, SQLite, blob dir.
- `desktop/src-tauri/src/sync/`: encryption, metadata, push, pull, apply, engine.
- Tauri commands: `configure_sync`, `sync_now`, `get_sync_status`, `disconnect_sync`.
