#!/usr/bin/env bash
set -euo pipefail

# Always run from the script's directory
cd "$(dirname "$0")"

# Ensure production mode for tools that rely on NODE_ENV
export NODE_ENV=production

# Preload .env.production so dotenv (which doesn't override by default)
# will keep these values when orval.config.ts calls dotenv.config()
if [ -f ".env.production" ]; then
	set -a
	# shellcheck disable=SC1091
	. ./.env.production
	set +a
fi

echo "Generating SDK in production mode..."
bun generate:sdk

echo "Building Tauri app (bundles: app, target: universal-apple-darwin)..."
bun tauri build --bundles app --target universal-apple-darwin
