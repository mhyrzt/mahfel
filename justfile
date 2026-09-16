set dotenv-load

default:
    @just --list

# Install frontend dependencies using the pinned Bun lockfile.
bootstrap:
    just --justfile frontend/justfile install

# Run the Rust services directly from the host for focused development.
dev-api:
    just --justfile backend/justfile dev-api

# Apply pending SQLx migrations to the configured database.
migrate:
    just --justfile backend/justfile migrate

dev-media:
    just --justfile backend/justfile dev-media

dev-worker:
    just --justfile backend/justfile dev-worker

dev-web:
    just --justfile frontend/justfile dev

# Check Rust and frontend formatting.
fmt:
    just --justfile backend/justfile fmt
    just --justfile frontend/justfile fmt

lint:
    just --justfile backend/justfile lint
    just --justfile frontend/justfile lint
    just --justfile frontend/justfile typecheck

# Run the complete application test suites.
test:
    just --justfile backend/justfile test
    just --justfile frontend/justfile test

# Run Rust documentation tests separately from Nextest.
test-doc:
    just --justfile backend/justfile test-doc

# Run Rust unit tests only.
test-unit:
    just --justfile backend/justfile test-unit

# Run Rust integration tests only.
test-integration:
    just --justfile backend/justfile test-integration

# Format SQL migrations through the backend Justfile.
sql-fmt:
    just --justfile backend/justfile sql-fmt

sql-lint:
    just --justfile backend/justfile sql-lint

# Start the disposable PostgreSQL integration test environment.
test-database:
    just --justfile backend/justfile test-database

# Compile the Rust workspace and frontend without quality checks.
build:
    just --justfile backend/justfile build
    just --justfile frontend/justfile build

# Delegate a focused backend recipe.
backend *args:
    just --justfile backend/justfile {{args}}

# Delegate a focused frontend recipe.
frontend *args:
    just --justfile frontend/justfile {{args}}

# Build the strict MkDocs site.
docs:
    mkdocs build --strict

# Start the local MkDocs development server.
docs-serve:
    mkdocs serve

# Start all local application, infrastructure, and observability profiles.
compose-up:
    docker compose --profile mahfel --profile infrastructure --profile observability up --build -d
    just trust-local-ca

# Stop the local Compose stack. Named volumes are retained.
compose-down:
    docker compose down

# Follow logs from all local Compose profiles.
compose-logs:
    docker compose --profile mahfel --profile infrastructure --profile observability logs -f

# Install and trust Caddy's generated local HTTPS root certificate.
trust-local-ca:
    #!/usr/bin/env bash
    set -euo pipefail
    certificate_dir=".local/caddy"
    certificate_path="${certificate_dir}/root.crt"
    mkdir -p "${certificate_dir}"
    for attempt in {1..30}; do
        if docker compose cp caddy:/data/caddy/pki/authorities/local/root.crt "${certificate_path}"; then
            break
        fi
        if [[ "${attempt}" == 30 ]]; then
            echo "Caddy local root CA was not ready after 30 seconds" >&2
            exit 1
        fi
        sleep 1
    done
    case "$(uname -s)" in
        Linux)
            if command -v update-ca-trust >/dev/null 2>&1; then
                sudo install -m 0644 "${certificate_path}" /etc/pki/ca-trust/source/anchors/mahfel-caddy-root.crt
                sudo update-ca-trust
            elif command -v update-ca-certificates >/dev/null 2>&1; then
                sudo install -m 0644 "${certificate_path}" /usr/local/share/ca-certificates/mahfel-caddy-root.crt
                sudo update-ca-certificates
            else
                echo "Unsupported Linux trust store; install ${certificate_path} manually" >&2
                exit 1
            fi
            ;;
        Darwin)
            sudo security add-trusted-cert -d -r trustRoot \
                -k /Library/Keychains/System.keychain "${certificate_path}"
            ;;
        *)
            echo "Unsupported host OS; install ${certificate_path} manually" >&2
            exit 1
            ;;
    esac
    echo "Trusted Mahfel local CA. Restart open browsers before using https://mahfel.localhost"
