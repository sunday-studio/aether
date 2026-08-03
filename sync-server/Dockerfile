FROM rust:1-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/aether-sync-server /aether-sync-server
EXPOSE 8080
VOLUME /data
ENV DATA_ROOT=/data
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 CMD ["/aether-sync-server", "healthcheck"]
CMD ["/aether-sync-server"]
