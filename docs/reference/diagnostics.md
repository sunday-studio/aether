# Diagnostics and debug logs

Aether keeps diagnostics local by default. The desktop app records bounded timing ledgers for local fetch paths and exposes a Settings export action that writes a redacted JSON debug log on the user's device.

## What is captured

- Frontend API-client timings: route matching, body parsing, argument building, Tauri invoke time, status, route, method, and body byte count.
- Rust command timings: command boundary elapsed time, DB gate wait, repository elapsed time, result count, pagination mode, and hydration phases where practical.
- Rust repository timings: local SQL connection and query/read-loop timings for entry and task fetch paths.

## Privacy rules

Debug logs should not include journal or task body content by default. They keep counts, IDs, route names, elapsed times, and byte sizes instead.

Known sensitive keys are redacted before persistence and again during export, including passphrases, passwords, tokens, API keys, authorization values, credentials, and private keys. Frontend URLs preserve paths and safe numeric pagination values, but redact query values because search strings can contain user content.

## Export path

Users can export diagnostics from `Settings -> Diagnostics -> Export`. The command writes a JSON file under the local diagnostics directory and returns the path in the UI toast and section body.

For local development, Rust timing ledgers live under `desktop/src-tauri/target/diagnostics/`. On macOS, production exports are written to `~/Library/Application Support/com.cas.aether/diagnostics/`.

## Live error log

`aether-errors.jsonl` is a bounded, append-only local ledger of redacted operational failures. Each line is a JSON object with a timestamp, component, message, and safe diagnostic details. The Diagnostics settings section can create and open the file. It records all Tauri command failures, Rust panics, and every `tracing::error!` event. Sync pull and push failures additionally retain the HTTP status and redacted response body.

## In-house vs PostHog

Recommendation: keep production debug-log export in-house for now. Aether is a local-first journaling and task app, so the default support path should be user-initiated export rather than automatic remote telemetry.

PostHog is useful if the product later needs aggregate product analytics, error tracking, session replay, or centralized logs. Its current logs product accepts OpenTelemetry log records, and its privacy docs put collection responsibility on the app team while offering controls for collection, processing, and storage. Session replay has masking controls, but those controls still need deliberate implementation in the app.

The weak part of adopting PostHog now is not technical integration. It is consent, expectation, and data-minimization risk. A journaling app can accidentally turn diagnostic telemetry into sensitive behavioral or content telemetry. If PostHog is added later, it should be opt-in, disabled by default, and limited to coarse events and crash/error metadata with content fields excluded at the source.

Near-term model:

- Use in-house local debug export for slow local fetches and production support.
- Add PostHog only after there is a clear product analytics question that cannot be answered with support exports or local smoke tests.
- Treat remote logs, session replay, and exception capture as separate consent surfaces, not one broad telemetry toggle.
