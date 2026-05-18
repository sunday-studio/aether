SHELL := /bin/sh

DESKTOP_DIR := desktop
DESKTOP_TAURI_DIR := desktop/src-tauri
SYNC_SERVER_DIR := sync-server

PNPM := pnpm
CARGO := cargo
DOCKER_COMPOSE := docker compose

.DEFAULT_GOAL := help

.PHONY: help
help:
	@printf "Aether development commands\n\n"
	@printf "Setup\n"
	@printf "  make setup              Install/fetch dependencies for every app\n"
	@printf "  make setup-desktop      Install desktop frontend dependencies\n"
	@printf "  make setup-rust         Fetch Rust dependencies for desktop backend and sync server\n"
	@printf "  make setup-sync-server  Fetch sync-server Rust dependencies\n\n"
	@printf "Development\n"
	@printf "  make dev                Run the desktop app through Tauri\n"
	@printf "  make dev-web            Run the desktop frontend only\n"
	@printf "  make dev-sync-server    Run the sync server locally\n"
	@printf "  make sync-server-up     Run sync server with Docker Compose\n"
	@printf "  make sync-server-down   Stop sync server Docker Compose stack\n\n"
	@printf "Build and verification\n"
	@printf "  make build              Build desktop frontend and sync server\n"
	@printf "  make check              Run frontend lint, desktop Rust check, and sync-server check\n"
	@printf "  make test               Run Rust tests for desktop backend and sync server\n"
	@printf "  make format             Format desktop frontend code\n"
	@printf "  make format-check       Check desktop frontend formatting\n"
	@printf "  make lint               Run desktop frontend lint\n\n"
	@printf "Generated API\n"
	@printf "  make generate-openapi   Generate desktop OpenAPI spec from Rust commands\n"
	@printf "  make generate-sdk       Generate OpenAPI spec and TypeScript SDK\n"

.PHONY: setup setup-desktop setup-rust setup-sync-server
setup: setup-desktop setup-rust

setup-desktop:
	cd $(DESKTOP_DIR) && $(PNPM) install

setup-rust:
	cd $(DESKTOP_TAURI_DIR) && $(CARGO) fetch
	cd $(SYNC_SERVER_DIR) && $(CARGO) fetch

setup-sync-server:
	cd $(SYNC_SERVER_DIR) && $(CARGO) fetch

.PHONY: dev dev-web dev-desktop dev-sync-server
dev: dev-desktop

dev-desktop:
	cd $(DESKTOP_DIR) && $(PNPM) run tauri:dev

dev-web:
	cd $(DESKTOP_DIR) && $(PNPM) run dev

dev-sync-server:
	cd $(SYNC_SERVER_DIR) && DATA_ROOT=./data $(CARGO) run

.PHONY: build build-desktop build-desktop-app build-sync-server
build: build-desktop build-sync-server

build-desktop:
	cd $(DESKTOP_DIR) && $(PNPM) run build

build-desktop-app:
	cd $(DESKTOP_DIR) && $(PNPM) run tauri -- build

build-sync-server:
	cd $(SYNC_SERVER_DIR) && $(CARGO) build

.PHONY: check check-desktop check-desktop-rust check-sync-server
check: lint check-desktop-rust check-sync-server

check-desktop: lint check-desktop-rust

check-desktop-rust:
	cd $(DESKTOP_TAURI_DIR) && $(CARGO) check

check-sync-server:
	cd $(SYNC_SERVER_DIR) && $(CARGO) check

.PHONY: test test-desktop-rust test-sync-server
test: test-desktop-rust test-sync-server

test-desktop-rust:
	cd $(DESKTOP_TAURI_DIR) && $(CARGO) test

test-sync-server:
	cd $(SYNC_SERVER_DIR) && $(CARGO) test

.PHONY: lint format format-check
lint:
	cd $(DESKTOP_DIR) && $(PNPM) run lint

format:
	cd $(DESKTOP_DIR) && $(PNPM) run format

format-check:
	cd $(DESKTOP_DIR) && $(PNPM) run format:check

.PHONY: generate-openapi generate-sdk
generate-openapi:
	cd $(DESKTOP_TAURI_DIR)/tools && $(CARGO) run --bin generate-openapi

generate-sdk: generate-openapi
	cd $(DESKTOP_DIR) && $(PNPM) run generate:sdk

.PHONY: sync-server-up sync-server-down sync-server-logs sync-server-docker-build
sync-server-up:
	cd $(SYNC_SERVER_DIR) && $(DOCKER_COMPOSE) up --build

sync-server-down:
	cd $(SYNC_SERVER_DIR) && $(DOCKER_COMPOSE) down

sync-server-logs:
	cd $(SYNC_SERVER_DIR) && $(DOCKER_COMPOSE) logs -f

sync-server-docker-build:
	cd $(SYNC_SERVER_DIR) && docker build -t aether-sync-server:local .
