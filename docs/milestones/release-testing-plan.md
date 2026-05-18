# Release Testing Plan

This plan defines the checks to run before cutting a v1 desktop release. It should be updated as release automation matures.

## Release Inputs

- Confirm `desktop/src-tauri/tauri.conf.json` has the intended release version.
- Confirm `desktop/src-tauri/Cargo.toml` has the same desktop package version.
- Confirm `desktop/package.json` has the expected frontend package version.
- Confirm the Tauri updater public key in `desktop/src-tauri/tauri.conf.json` matches the private key stored in release secrets.
- Confirm `TAURI_SIGNING_PRIVATE_KEY` is available to release builds as a secret, either as the private key contents or a path to the private key file.
- Confirm `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` is available if the private key is password-protected.
- Confirm GitHub release permissions can upload `latest.json` and platform artifacts.

## Static Checks

- Run `pnpm install --frozen-lockfile` from `desktop/`.
- Run `pnpm run lint` from `desktop/`.
- Run `pnpm run build` from `desktop/`.
- Run `cargo check` from `desktop/src-tauri/`.
- Run `cargo check` from `sync-server/`.
- Confirm no local-only updater endpoint is committed.
- Confirm no updater private key file is committed.

## Desktop Smoke Checks

- Launch a clean app profile.
- Complete first-run onboarding.
- Restart and confirm onboarding does not reappear.
- Create a journal entry.
- Create a task.
- Create a goal.
- Open Settings and verify Preferences, Sync, AI, and What's New.
- Confirm Canvas, Bookmarks, Graph, and placeholder routes are not exposed in v1 navigation.
- Confirm `[[...]]` resource-link autocomplete does not offer hidden resource types.

## Sync Smoke Checks

- Configure sync with server URL, server seed phrase, and sync passphrase.
- Run manual sync.
- Restart the app.
- Reconnect with sync passphrase.
- Confirm pending changes are pushed.
- Confirm pulled changes appear locally.
- Confirm sync failure copy is understandable and not debug-only.

## Updater Local Test

Use this when validating the updater before publishing a real GitHub release.

1. Generate or retrieve the updater signing key.
2. Put the public key in `desktop/src-tauri/tauri.conf.json`.
3. Store the private key outside the repo.
4. Build and install an older version, for example `0.1.0`.
5. Bump the app version to a newer version, for example `0.1.1`.
6. Build signed updater artifacts using `TAURI_SIGNING_PRIVATE_KEY`.
7. Serve `latest.json` and the update artifact from a local HTTP server or a draft GitHub release.
8. Temporarily point the updater endpoint at the local `latest.json`.
9. Launch the installed older app.
10. Confirm the top-right `New update` button appears.
11. Click `New update`.
12. Confirm Settings opens directly to What's New.
13. Confirm the release notes and target version are visible.
14. Click install and confirm download progress updates.
15. Confirm the app restarts into the new version.
16. Confirm skipped versions remain skipped after restart.
17. Restore the production updater endpoint before committing.

## Updater GitHub Release Test

Use this before marking a GitHub release ready.

1. Build signed release artifacts in CI.
2. Confirm `latest.json` is uploaded to the GitHub release.
3. Confirm platform artifacts referenced by `latest.json` are uploaded.
4. Confirm `latest.json` version is newer than the installed app.
5. Install the previous released version.
6. Launch the previous version.
7. Run a manual update check in Settings > What's New.
8. Confirm the global `New update` button appears after an update check or focus check.
9. Confirm install downloads and restarts into the new version.
10. Confirm the old version no longer reports an update after restart.

## Release Acceptance

- The app builds without TypeScript or Rust errors.
- OXC lint exits with zero errors.
- Known lint warnings are either fixed or documented.
- The updater can discover a newer signed version.
- The updater can install that version and restart.
- The release does not expose private signing keys, local endpoints, debug routes, or placeholder product surfaces.
