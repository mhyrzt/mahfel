FROM docker.arvancloud.ir/rust:1.95.0-bookworm AS builder
ENV RUSTUP_TOOLCHAIN=1.95.0-x86_64-unknown-linux-gnu \
    CARGO_HTTP_TIMEOUT=600 \
    CARGO_HTTP_LOW_SPEED_LIMIT=1 \
    CARGO_NET_RETRY=10
WORKDIR /source
COPY backend/ ./backend/
COPY proto/ ./proto/
WORKDIR /source/backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/source/backend/target \
    cargo build --locked --release --bin mahfel-api && \
    cp target/release/mahfel-api /tmp/mahfel-api

FROM docker.arvancloud.ir/debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
WORKDIR /app
COPY --from=builder /tmp/mahfel-api /usr/local/bin/mahfel-api
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 8080 9101
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=5 \
  CMD curl --fail --silent http://127.0.0.1:9101/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-api"]
