set dotenv-load

default:
    @just --list

bootstrap:
    just --justfile frontend/justfile install

dev-api:
    just --justfile backend/justfile dev-api

migrate:
    just --justfile backend/justfile migrate

dev-media:
    just --justfile backend/justfile dev-media

dev-worker:
    just --justfile backend/justfile dev-worker

dev-web:
    just --justfile frontend/justfile dev

fmt:
    just --justfile backend/justfile fmt
    just --justfile frontend/justfile fmt

lint:
    just --justfile backend/justfile lint
    just --justfile frontend/justfile lint
    just --justfile frontend/justfile typecheck

test:
    just --justfile backend/justfile test
    just --justfile frontend/justfile test

test-doc:
    just --justfile backend/justfile test-doc

sql-fmt:
    just --justfile backend/justfile sql-fmt

sql-lint:
    just --justfile backend/justfile sql-lint

test-database:
    just --justfile backend/justfile test-database

build:
    just --justfile backend/justfile build
    just --justfile frontend/justfile build

backend *args:
    just --justfile backend/justfile {{args}}

frontend *args:
    just --justfile frontend/justfile {{args}}

docs:
    mkdocs build --strict

docs-serve:
    mkdocs serve

compose-up:
    docker compose up --build -d
    just trust-local-ca

compose-down:
    docker compose down

compose-logs:
    docker compose logs -f

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
