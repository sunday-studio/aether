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

# Set cross-compiler
ENV CC=aarch64-linux-gnu-gcc

# Set linker flags to handle duplicate symbols (both libSQL and GORM SQLite driver include SQLite)
ENV CGO_LDFLAGS="-Wl,--allow-multiple-definition"

# Build ARM64 binary with CGO enabled
RUN GOOS=linux GOARCH=arm64 CGO_ENABLED=1 go build -ldflags="-Wl,--allow-multiple-definition" -o /src/aether-backend_${VERSION} ./main.go
