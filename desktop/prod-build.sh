#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

export NODE_ENV=production

if [ -f ".env.production" ]; then
	set -a
	. ./.env.production
	set +a
fi

echo "Step 1: Generating OpenAPI spec..."
cd src-tauri
cargo run --bin generate-openapi -p generate-openapi-tool
cd ..

echo "Step 2: Generating SDK from OpenAPI spec..."
bun generate:sdk

echo "Step 3: Building Tauri app (universal binary for macOS)..."
bun tauri build --bundles app --target aarch64-apple-darwin

echo "✅ Build complete!"