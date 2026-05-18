# Aether Sync Server

E2E-encrypted sync server for Aether. Stores opaque encrypted changes and blobs; no plaintext.

## Endpoints

- `GET /health` – health check
- `POST /push` – `{ device_id, changes: [{ nonce, ciphertext }] }`
- `GET /pull?since=ts` – `{ changes, timestamp, has_more }`
- `PUT /media/:hash` – upload blob
- `GET /media/:hash` – download blob
- `HEAD /media/:hash` – blob exists

## Run

```bash
DATA_ROOT=./data cargo run
```

## Docker

```bash
docker compose up --build
```

Data in `./data` (sync.db and blobs/).
