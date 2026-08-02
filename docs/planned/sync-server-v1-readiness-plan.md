# Sync Server V1 Readiness Plan

**Status:** Planned. This document records v1 follow-up work; it does not change the current service behaviour.

## Scope and decision boundary

The sync server remains a single-user, self-hosted relay for encrypted Aether data. Enrolled devices share one server namespace. Multi-tenant accounts, tenant isolation, and an administrative UI are outside this plan unless a future product decision changes that model.

## Planned follow-up work

### Transport policy

- Require HTTPS for every non-loopback deployment and reject or prominently warn about insecure remote server URLs in clients.
- Publish a supported reverse-proxy/TLS deployment example, including certificate renewal and WebSocket proxy settings.
- Keep server seed phrases and device bearer tokens out of logs, URLs, and unencrypted configuration files.

### Backup, restore, and device recovery

- Document a backup procedure that captures both `sync.db` and the `blobs/` directory from the same data root.
- Provide a verified restore procedure, including a post-restore readiness check and a test-device pull.
- Add an owner-controlled device revocation path that invalidates a lost device token.
- Define re-enrolment and token rotation so a recovered or replacement device can join without weakening the seed-phrase boundary.

### Retention and compaction

- Define the maximum retained change history and blob policy before enabling deletion or compaction.
- Design snapshots/checkpoints that let a newly enrolled device reconstruct state after old changes are pruned.
- Define the offline-device grace period, recovery flow after that period, and user-visible warning before a device requires reset.
- Keep compaction conservative until the client can prove a snapshot is safely applicable.

### Bounded media resource use

- Replace whole-file request buffering with streamed uploads and downloads.
- Set explicit, documented media size limits and return actionable errors when exceeded.
- Bound concurrent media transfers and storage growth so one enrolled device cannot exhaust server memory or disk.
- Preserve encrypted blob semantics while adding integrity checks and cleanup for interrupted uploads.

## Acceptance criteria before these items ship

- A remote deployment cannot silently operate over plaintext transport.
- Operators can restore a complete data root and verify it before clients reconnect.
- A lost device can be revoked and re-enrolled without replacing every healthy device.
- Retention never deletes history that an allowed offline device still requires without an explicit recovery path.
- Media transfer memory use is bounded independently of individual file size.
