FROM docker.arvancloud.ir/rust:1.95.0-bookworm AS chef
ENV CARGO_HTTP_TIMEOUT=600 \
    CARGO_HTTP_LOW_SPEED_LIMIT=1 \
    CARGO_NET_RETRY=10
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates clang cmake g++ make pkg-config python3 && \
    rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends python3-pip && \
    rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt --toolchain 1.95.0 && rustfmt --version
RUN cargo install cargo-chef --version 0.1.78 --locked
WORKDIR /source

FROM chef AS planner
COPY backend/ ./backend/
COPY proto/ ./proto/
WORKDIR /source/backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher
WORKDIR /source/backend
COPY --from=planner /source/backend/recipe.json recipe.json
RUN --mount=type=cache,id=mahfel-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-target,target=/source/backend/target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

FROM cacher AS api-builder
COPY backend/ ./
COPY proto/ ../proto/
RUN --mount=type=cache,id=mahfel-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-target,target=/source/backend/target,sharing=locked \
    cargo build --locked --release --bin mahfel-api && \
    cp target/release/mahfel-api /tmp/mahfel-api

FROM cacher AS media-builder
COPY backend/ ./
COPY proto/ ../proto/
RUN --mount=type=cache,id=mahfel-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-target,target=/source/backend/target,sharing=locked \
    cargo build --locked --release --bin mahfel-media && \
    cp target/release/mahfel-media /tmp/mahfel-media

FROM cacher AS worker-builder
COPY backend/ ./
COPY proto/ ../proto/
RUN --mount=type=cache,id=mahfel-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=mahfel-cargo-target,target=/source/backend/target,sharing=locked \
    cargo build --locked --release --bin mahfel-worker && \
    cp target/release/mahfel-worker /tmp/mahfel-worker

FROM docker.arvancloud.ir/debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
WORKDIR /app
COPY --from=api-builder /tmp/mahfel-api /usr/local/bin/mahfel-api
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
COPY --chown=mahfel:mahfel backend/config/docker.toml /app/config/docker.toml
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 8080 9101
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=5 \
  CMD curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9101/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-api"]

FROM docker.arvancloud.ir/debian:bookworm-slim AS media
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl libgcc-s1 libstdc++6 && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
WORKDIR /app
COPY --from=media-builder /tmp/mahfel-media /usr/local/bin/mahfel-media
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
COPY --chown=mahfel:mahfel backend/config/docker.toml /app/config/docker.toml
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 50051 9102 40000/udp 40000/tcp 40001/udp 40001/tcp 40002/udp 40002/tcp 40003/udp 40003/tcp
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
  CMD curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9102/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-media"]

FROM docker.arvancloud.ir/debian:bookworm-slim AS worker
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
WORKDIR /app
COPY --from=worker-builder /tmp/mahfel-worker /usr/local/bin/mahfel-worker
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
COPY --chown=mahfel:mahfel backend/config/docker.toml /app/config/docker.toml
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 9103
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=5 \
  CMD curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9103/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-worker"]
