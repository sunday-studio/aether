# V1 Release Checklist

This is the working list to run before cutting v1. The intent is to ship a smaller, coherent app with no placeholder surfaces.

## 1. Lock V1 Product Surface

- Keep Journal.
- Keep Tasks.
- Keep Goals.
- Keep Settings.
- Keep self-hosted encrypted Sync.
- Keep Updater.
- Keep command palette search.
- Keep local search model setup for semantic search.
- Keep resource links if they stay scoped to v1-ready resources.
- Hide journal audio recording and transcription for v1 until the flow is stable.
- Hide Canvas for v1.
- Hide Bookmarks for v1.
- Hide Graph for v1 unless the visualization and route behavior are finished.
- Hide backend-only diagnostic and transcription model-management features unless there is a complete user-facing flow.

## 2. First-Run Onboarding

- Add a first-launch onboarding gate before the main app.
- Collect basic user profile data, starting with display name.
- Offer optional AI setup during onboarding.
- Store OpenAI and Groq keys through the existing encrypted settings path.
- Let users choose a default AI provider.
- Allow users to skip AI setup without blocking app use.
- Persist onboarding completion in settings, for example `app.onboarding_completed`.
- Route returning users directly to the main app.

## 3. AI And Transcription Readiness

- Make configured AI provider status visible.
- Make local search model download/status visible.
- Validate configured provider credentials from the UI.
- Keep journal audio recording and transcription entry points hidden for v1.
- Leave audio/transcription backend code available for a later stabilization pass.
- Keep local embedding model setup scoped to search.
- Leave local transcription model management backend-only for v1.

## 4. Updater

- Keep updater in v1.
- Keep Settings > What's New.
- Keep the global update indicator wired to updater state.
- Verify manual update checks.
- Verify update available state.
- Verify skip version.
- Verify download and install.
- Verify update preferences persist.
- Verify auto-check behavior if enabled.
- Run the updater section of the [release testing plan](./release-testing-plan.md).

## 5. Sync

- Keep self-hosted encrypted sync in v1.
- Verify setup with server URL, server seed phrase, and sync passphrase.
- Verify reconnect with sync passphrase.
- Verify manual sync.
- Verify periodic or websocket-triggered sync if enabled.
- Verify media sync policy: auto and on-demand.
- Hide sync diagnostics from user-facing settings.
- Keep sync failure messages understandable and non-debuggy.

## 6. Navigation And App Shell Cleanup

- Remove or disable the `/canvas` route for v1.
- Remove the Canvas keyboard shortcut.
- Remove Canvas destinations from command palette and resource navigation.
- Remove Bookmarks and Graph destinations from command palette and resource navigation.
- Hide placeholder routes.
- Ensure bottom navigation exposes only v1 features.
- Ensure command palette search only opens v1-ready destinations.
- Ensure app startup has a clear path through onboarding or the main app.

## 7. Backend-Frontend Gap Cleanup

- Decide for each backend capability whether it is exposed, hidden, or deferred.
- Bookmarks: backend exists, frontend is currently placeholder-level, so keep hidden for v1.
- Search: command palette search is exposed for v1; full search-results pages remain later work.
- Embeddings: local search model setup is exposed for search; transcription model management remains deferred.
- Transcription provider/model management: backend exists, frontend exposure is partial, so keep model management hidden and keep journal audio/transcription UI disabled for v1 while AI key setup stays visible.
- Sync diagnostics: backend exists, keep hidden for v1.

## 8. Release Polish

- Add useful empty states for visible routes.
- Check loading states.
- Check error states.
- Remove visible TODO-like UX.
- Remove dead buttons.
- Remove routes that land on placeholder screens.
- Confirm settings copy distinguishes sync passphrase from server seed phrase.
- Confirm AI key copy explains what is optional.

## 9. Verification

- Run `cargo check` in `desktop/src-tauri`.
- Run `cargo check` in `sync-server`.
- Run the frontend typecheck or build for `desktop`.
- Smoke first-launch onboarding.
- Smoke journal entry creation.
- Smoke task and goal creation.
- Smoke sync configure, sync now, reconnect, and media policy.
- Smoke updater settings and manual check.
- Smoke updater discovery, global update button, signed download, install, and restart.
- Smoke local search model download and embedding rebuild.
- Smoke command palette hybrid search.
- Restart the app and confirm persisted onboarding/settings state.
