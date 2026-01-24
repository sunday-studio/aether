FROM rust:1-bookworm AS build
WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/aether-sync-server /aether-sync-server
EXPOSE 8080
VOLUME /data
ENV DATA_ROOT=/data
CMD ["/aether-sync-server"]
