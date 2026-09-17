set dotenv-load

default:
    @just --list

# Create local environment files without overwriting existing values, then initialize Caddy's
# persistent local HTTPS certificate and trust it on the host.
bootstrap:
    #!/usr/bin/env bash
    set -euo pipefail

    copy_if_missing() {
        local source="$1"
        local destination="$2"
        if [[ ! -f "${destination}" ]]; then
            cp "${source}" "${destination}"
            echo "Created ${destination}"
        else
            echo "Keeping existing ${destination}"
        fi
    }

    copy_if_missing .env.example .env
    copy_if_missing frontend/.env.local.example frontend/.env.local
    just caddy-init

# Start infrastructure, including Caddy, without building application images.
caddy-init:
    docker compose --profile infrastructure up -d
    just trust-local-ca

# Forward arbitrary recipes and arguments to the backend Justfile.
backend *args:
    just --justfile backend/justfile {{args}}

# Forward arbitrary recipes and arguments to the frontend Justfile.
frontend *args:
    just --justfile frontend/justfile {{args}}

# Forward arbitrary recipes and arguments to the protobuf Justfile.
proto *args:
    just --justfile proto/justfile {{args}}

# Build the strict MkDocs site.
docs:
    mkdocs build --strict

# Start the local MkDocs development server.
docs-serve:
    mkdocs serve

# Start selected Compose profiles; no arguments starts all profiles.
compose-up *profiles:
    #!/usr/bin/env bash
    set -euo pipefail
    selected=("{{profiles}}")
    if [[ -z "${selected[*]}" ]]; then selected=(mahfel infrastructure observability); fi
    flags=()
    for profile in "${selected[@]}"; do flags+=(--profile "$profile"); done
    docker compose "${flags[@]}" up --build -d
    if printf '%s\n' "${selected[@]}" | grep -qx mahfel; then just trust-local-ca; fi

# Stop the local Compose stack. Named volumes are retained.
compose-down:
    docker compose down

# Follow logs from selected Compose profiles; no arguments means all profiles.
compose-logs *profiles:
    #!/usr/bin/env bash
    set -euo pipefail
    selected=("{{profiles}}")
    if [[ -z "${selected[*]}" ]]; then selected=(mahfel infrastructure observability); fi
    flags=()
    for profile in "${selected[@]}"; do flags+=(--profile "$profile"); done
    docker compose "${flags[@]}" logs -f

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
