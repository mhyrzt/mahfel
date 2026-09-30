FROM docker.arvancloud.ir/rust:1.95.0-bookworm AS chef
ENV CARGO_HTTP_TIMEOUT=600 \
    CARGO_HTTP_LOW_SPEED_LIMIT=1 \
    CARGO_NET_RETRY=10
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates clang cmake g++ make pkg-config python3 \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev && \
    rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y --no-install-recommends python3-pip && \
    rm -rf /var/lib/apt/lists/*
RUN rustup component add clippy rustfmt --toolchain 1.95.0 && \
    clippy-driver --version && rustfmt --version
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

FROM chef AS ci-integration
ARG CARGO_NEXTEST_VERSION=0.9.146
RUN cargo install cargo-nextest --version "$CARGO_NEXTEST_VERSION" --locked
RUN apt-get update && apt-get install -y --no-install-recommends \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly gstreamer1.0-libav && rm -rf /var/lib/apt/lists/*
COPY backend/ /source/backend/
COPY proto/ /source/proto/
COPY deploy/keycloak/ /source/deploy/keycloak/
WORKDIR /source/backend
RUN cargo nextest run --locked --workspace --all-features --tests --no-run
CMD ["cargo", "nextest", "run", "--locked", "--profile", "ci", "--workspace", "--all-features", "--tests", "--no-capture"]

FROM docker.arvancloud.ir/debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
WORKDIR /app
COPY --from=api-builder /tmp/mahfel-api /usr/local/bin/mahfel-api
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
COPY --chown=mahfel:mahfel backend/config/docker.toml /app/config/docker.toml
COPY --chown=mahfel:mahfel backend/seeds/ /app/seeds/
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 8080 9101
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=5 \
  CMD curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9101/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-api"]

FROM docker.arvancloud.ir/debian:bookworm-slim AS media
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl libgcc-s1 libstdc++6 \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
RUN mkdir -p /tmp/mahfel-recordings && chown mahfel:mahfel /tmp/mahfel-recordings
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

FROM docker.arvancloud.ir/oven/bun:1.4.2 AS recording-renderer-builder
WORKDIR /source/frontend
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
RUN bun run recording-renderer:build
ENV PLAYWRIGHT_BROWSERS_PATH=/app/browsers
RUN bunx playwright-core install chromium

FROM docker.arvancloud.ir/debian:bookworm-slim AS worker-runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav \
    libnss3 libatk1.0-0 libatk-bridge2.0-0 libx11-6 libxcomposite1 libxdamage1 \
    libxrandr2 libgbm1 libasound2 libpango-1.0-0 libcairo2 libxkbcommon0 libatspi2.0-0 && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system --gid 10001 mahfel && \
    useradd --system --uid 10001 --gid mahfel --home-dir /nonexistent --shell /usr/sbin/nologin mahfel
RUN mkdir -p /tmp/mahfel-exports /tmp/mahfel-browser && chown mahfel:mahfel /tmp/mahfel-exports /tmp/mahfel-browser
COPY --from=recording-renderer-builder /usr/local/bin/bun /usr/local/bin/bun
COPY --from=recording-renderer-builder /app/browsers /app/browsers
COPY --from=recording-renderer-builder /source/frontend/recording-renderer-dist /app/recording-renderer
COPY --from=recording-renderer-builder /source/frontend/node_modules/playwright-core /app/recording-renderer/node_modules/playwright-core
ENV PLAYWRIGHT_BROWSERS_PATH=/app/browsers HOME=/tmp/mahfel-browser
WORKDIR /app

FROM worker-runtime AS worker
COPY --from=worker-builder /tmp/mahfel-worker /usr/local/bin/mahfel-worker
COPY --chown=mahfel:mahfel backend/config/default.toml /app/config/default.toml
COPY --chown=mahfel:mahfel backend/config/docker.toml /app/config/docker.toml
ENV MAHFEL_CONFIG_DIR=/app/config
USER mahfel
EXPOSE 9103
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=5 \
  CMD curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9103/health/live >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/mahfel-worker"]
