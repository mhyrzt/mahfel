FROM rust:1.95.0-bookworm AS builder
ENV RUSTUP_TOOLCHAIN=1.95.0-x86_64-unknown-linux-gnu \
    CARGO_HTTP_TIMEOUT=600 \
    CARGO_HTTP_LOW_SPEED_LIMIT=1 \
    CARGO_NET_RETRY=10
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates clang cmake curl g++ make pkg-config python3 python3-pip && \
    rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt --toolchain "$RUSTUP_TOOLCHAIN" && rustfmt --version
WORKDIR /source
COPY backend/ ./backend/
COPY proto/ ./proto/
WORKDIR /source/backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cache/pip \
    --mount=type=cache,target=/source/backend/target \
    cargo build --locked --release --bin mahfel-media && \
    cp target/release/mahfel-media /tmp/mahfel-media

FROM builder AS validation
RUN rustup component add clippy --toolchain "$RUSTUP_TOOLCHAIN" && clippy-driver --version
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cache/pip \
    --mount=type=cache,target=/source/backend/target \
    cargo fmt --all -- --check && \
    cargo check --workspace && \
    cargo clippy --workspace --all-targets --all-features -- -D warnings && \
    cargo test --workspace

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl libgcc-s1 libstdc++6 && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
WORKDIR /app
COPY --from=builder /tmp/mahfel-media /usr/local/bin/mahfel-media
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 50051 9102 40000/udp 40000/tcp 40001/udp 40001/tcp 40002/udp 40002/tcp 40003/udp 40003/tcp
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
  CMD curl --fail --silent http://127.0.0.1:9102/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-media"]
