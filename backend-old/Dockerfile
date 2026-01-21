# Stage 1: Build backend with ARM64 CGO
FROM golang:tip-bookworm AS builder

# Install ARM64 cross-compiler for CGO
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /src

# Copy Go modules
COPY go.mod go.sum ./

# Download dependencies
RUN go mod download

# Copy all source code
COPY . .

# Set build version
ARG VERSION=dev

# Environment variables as build args
ARG LIBSQL_URL
ARG LIBSQL_USE_REPLICA
ARG LIBSQL_SYNC_INTERVAL
ARG LIBSQL_AUTH_TOKEN

# Set cross-compiler
ENV CC=aarch64-linux-gnu-gcc

# Create .env file from build args
RUN echo "LIBSQL_URL=${LIBSQL_URL}" > /src/.env && \
    echo "LIBSQL_USE_REPLICA=${LIBSQL_USE_REPLICA}" >> /src/.env && \
    echo "LIBSQL_SYNC_INTERVAL=${LIBSQL_SYNC_INTERVAL}" >> /src/.env && \
    echo "LIBSQL_AUTH_TOKEN=${LIBSQL_AUTH_TOKEN}" >> /src/.env

# Build ARM64 binary with CGO enabled (required for libsql)
RUN GOOS=linux GOARCH=arm64 CGO_ENABLED=1 go build -o /src/aether-backend_${VERSION} ./main.go
