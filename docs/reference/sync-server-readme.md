# Aether Sync Server

E2E-encrypted sync server for Aether. Stores opaque encrypted changes and blobs; no plaintext.

## Endpoints

- `GET /health` – health check
- `POST /register` – enroll with `{ device_id, hostname, server_seed_phrase }`; returns a device token and encryption salt
- `POST /push` – authenticated encrypted change upload
- `GET /pull?cursor=received_at:change_id` – authenticated encrypted change download
- `GET /devices` – authenticated device list
- `GET /ws` – authenticated WebSocket sync notification stream
- `PUT /media/:hash` – upload blob
- `GET /media/:hash` – download blob
- `HEAD /media/:hash` – blob exists

## Run

```bash
SERVER_SEED_PHRASE="replace-with-a-long-random-server-seed" DATA_ROOT=./data cargo run
```

## Docker

```bash
docker compose up --build
```

Data in `./data` (sync.db and blobs/).
