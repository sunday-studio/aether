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

## Media sync

Media blobs (audio, images, etc.) are synced via `PUT/GET /media/{hash}`. The `content_hash` (`sha256:{hex}` of plaintext) is stored in `media_items.metadata.content_hash`.

**Setting: `sync.media_sync_policy`**

- **`auto`**: Blobs are uploaded when pushing `media_items` and downloaded when applying them. Full media is transferred during sync.
- **`on_demand`** (default): Only metadata is synced. Blobs are downloaded when the app needs them (e.g. playing audio). Use `ensure_media_blob(mediaId)` before loading media, or rely on `get_audio_data` which does it automatically.

Configure via Settings → Sync (Auto sync media / Download as needed) or `set_setting("sync.media_sync_policy", "auto"|"on_demand")`.

**Image/video:** When adding UI that displays image or video from `media_items`, call `ensure_media_blob(mediaId)` (or the `useEnsureMediaBlob(mediaId)` hook) before resolving a `file://` or blob URL when `sync.media_sync_policy` is `on_demand`.

## Development

- `sync-server/`: Axum server, SQLite, blob dir.
- `desktop/src-tauri/src/sync/`: encryption, metadata, push, pull, apply, engine, media.
- Tauri commands: `configure_sync`, `sync_now`, `get_sync_status`, `disconnect_sync`, `reconnect_sync`, `ensure_media_blob`.
- WebSocket: sync-server `/ws` broadcasts `"sync"` after push; desktop client runs sync on message.
