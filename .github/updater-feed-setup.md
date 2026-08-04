# Desktop updater feed setup

The desktop app reads only this stable feed:

```txt
https://sunday-studio.github.io/aether/updates/desktop/stable/latest.json
```

The first approved desktop release creates the `gh-pages` branch and configures
GitHub Pages to publish that branch root. No manual Pages setup is needed;
ensure GitHub Actions is allowed to create Pages sites for this repository. The
workflow then writes the manifest to `updates/desktop/stable/latest.json`.

The release workflow publishes immutable signed archives to the versioned GitHub
Release first. It promotes `latest.json` to Pages only after the GitHub Release
is public, so an incomplete release cannot become an offered app update.

The production environment must contain these release secrets:

- `TAURI_SIGNING_PRIVATE_KEY`;
- `APPLE_SIGNING_IDENTITY`.

It must also contain these release-only secrets:

- `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD`;
- `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, and `KEYCHAIN_PASSWORD`.

Set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` only when the private key is password-protected.

Use a Developer ID Application certificate. The build workflow rejects an Apple
Development fallback because it cannot support a public macOS auto-update.

For every release PR, commit the same version to `desktop/package.json`,
`desktop/src-tauri/Cargo.toml`, and `desktop/src-tauri/tauri.conf.json`. The
release preflight and platform builds refuse a mismatch or a release-note version
that differs from those files.
