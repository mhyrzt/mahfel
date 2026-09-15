set dotenv-load := true

default:
    @just --list

bootstrap:
    cd frontend && bun install --frozen-lockfile

dev-api:
    cd backend && cargo run --bin mahfel-api

migrate:
    cd backend && cargo run --bin mahfel-api -- migrate

dev-media:
    cd backend && cargo run --bin mahfel-media

dev-worker:
    cd backend && cargo run --bin mahfel-worker

dev-web:
    cd frontend && bun run dev

fmt:
    cd backend && cargo fmt --all -- --check

lint:
    cd backend && cargo clippy --workspace --all-targets --all-features -- -D warnings
    cd frontend && bun run lint
    cd frontend && bun run typecheck

test:
    cd backend && cargo test --workspace
    cd frontend && bun run test

test-database:
    docker compose up -d postgres
    cd backend && MAHFEL_TEST_DATABASE_URL="postgres://${MAHFEL_POSTGRES_USER:-mahfel}:${MAHFEL_POSTGRES_PASSWORD:-mahfel-local-only}@127.0.0.1:${MAHFEL_POSTGRES_HOST_PORT:-5432}/postgres" cargo test -p mahfel-database --test migrations -- --nocapture

build:
    cd backend && cargo build --workspace
    cd frontend && bun run build

docs:
    mkdocs build --strict

docs-serve:
    mkdocs serve

compose-up:
    docker compose up --build -d

compose-down:
    docker compose down

compose-logs:
    docker compose logs -f
