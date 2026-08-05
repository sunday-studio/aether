# Aether Mobile Shell

This is the isolated Tauri 2 mobile shell for Aether. It deliberately has no dependency on the desktop Rust crate or its frontend modules; Phase 1 will move genuinely cross-platform code into shared packages and crates.

## Local frontend validation

```sh
pnpm install
pnpm check
```

## Native mobile setup

Initialize each platform once on a machine with its Tauri prerequisites installed:

```sh
pnpm tauri:ios:init
pnpm tauri:android:init
```

Then run the target shell on a simulator, device, or emulator:

```sh
pnpm tauri:ios:dev
pnpm tauri:android:dev
```

Tauri passes `TAURI_DEV_HOST` to Vite for physical-device development. The Vite configuration binds to that host and exposes WebSocket HMR on a dedicated port.
