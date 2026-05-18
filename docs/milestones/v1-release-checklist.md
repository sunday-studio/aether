# V1 Release Checklist

This is the working list to run before cutting v1. The intent is to ship a smaller, coherent app with no placeholder surfaces.

## 1. Lock V1 Product Surface

- Keep Journal.
- Keep Tasks.
- Keep Goals.
- Keep Settings.
- Keep self-hosted encrypted Sync.
- Keep Updater.
- Keep audio transcription inside Journal if stable.
- Keep Graph/resource links only if navigation and empty states feel finished.
- Hide Canvas for v1.
- Hide Bookmarks unless the frontend is completed.
- Hide backend-only diagnostic and model-management features unless there is a complete user-facing flow.

## 2. First-Run Onboarding

- Add a first-launch onboarding gate before the main app.
- Collect basic user profile data, starting with display name.
- Offer AI setup during onboarding.
- Store OpenAI and Groq keys through the existing encrypted settings path.
- Let users choose a default transcription provider.
- Allow users to skip AI setup without blocking app use.
- Persist onboarding completion in settings, for example `app.onboarding_completed`.
- Route returning users directly to the main app.

## 3. AI And Transcription Readiness

- Make configured transcription provider status visible.
- Validate configured provider credentials from the UI.
- Confirm journal audio can be recorded or uploaded.
- Confirm transcription can be started from journal audio.
- Confirm transcript status moves through pending, processing, complete, and failed states.
- Defer full local model download UI unless it is intentionally added to v1.
- Leave embeddings management backend-only for v1.

## 4. Updater

- Keep updater in v1.
- Keep Settings > What's New.
- Re-enable or finish update notification events/toasts.
- Verify manual update checks.
- Verify update available state.
- Verify skip version.
- Verify download and install.
- Verify update preferences persist.
- Verify auto-check behavior if enabled.

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
- Hide placeholder routes.
- Ensure bottom navigation exposes only v1 features.
- Ensure command palette does not advertise unfinished destinations.
- Ensure app startup has a clear path through onboarding or the main app.

## 7. Backend-Frontend Gap Cleanup

- Decide for each backend capability whether it is exposed, hidden, or deferred.
- Bookmarks: backend exists, frontend is currently placeholder-level, so finish or hide.
- Search: backend exists, command palette search is commented out, so finish or hide.
- Embeddings: backend commands exist, no visible frontend management, so defer.
- Transcription provider/model management: backend exists, frontend exposure is partial, so keep only the pieces needed for onboarding and journal transcription.
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
- Smoke audio record or upload plus transcription.
- Smoke sync configure, sync now, reconnect, and media policy.
- Smoke updater settings and manual check.
- Restart the app and confirm persisted onboarding/settings state.

