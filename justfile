set dotenv-load

[group('General')]
default:
    @just --list

# Remove local build, test, Storybook, and documentation artifacts.
[group('Local development')]
clean:
    rm -rf backend/target frontend/.next frontend/out frontend/storybook-static frontend/coverage site

# Create local env files, start Caddy, and trust its HTTPS certificate.
[group('Local development')]
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
    just caddy

# Start only Caddy, without infrastructure or application dependencies.
[group('Local development')]
caddy:
    docker compose --profile caddy up -d caddy
    just trust-local-ca

# Forward arbitrary recipes and arguments to the backend Justfile.
[group('Components')]
backend *args:
    just --justfile backend/justfile {{args}}

# Forward arbitrary recipes and arguments to the frontend Justfile.
[group('Components')]
frontend *args:
    just --justfile frontend/justfile {{args}}

# Run cross-service browser tests from their independent package.
[group('Validation')]
e2e *args:
    cd e2e && bun run test {{args}}

# Forward arbitrary recipes and arguments to the protobuf Justfile.
[group('Components')]
proto *args:
    just --justfile proto/justfile {{args}}

# Build the strict MkDocs site.
[group('Documentation')]
docs:
    mkdocs build --strict

# Start the local MkDocs development server.
[group('Documentation')]
docs-serve:
    mkdocs serve

# Export the API-owned REST and WebSocket contracts.
[group('Contracts')]
api-docs-export:
    just backend api-docs-export

# Resolve requested Compose profiles and print the selected profiles, one per line.
_compose-selected-profiles *profiles:
    #!/usr/bin/env bash
    set -euo pipefail
    requested=("{{profiles}}")
    if [[ -z "${requested[*]}" ]]; then requested=(mahfel infrastructure observability); fi
    selected=()
    declare -A seen=()
    add_profile() {
        local profile="$1"
        if [[ -z "${seen[$profile]:-}" ]]; then
            selected+=("${profile}")
            seen[$profile]=1
        fi
    }
    for profile in "${requested[@]}"; do
        case "${profile}" in
            observability) add_profile infrastructure; add_profile mahfel ;;
            mahfel) add_profile infrastructure ;;
        esac
        add_profile "${profile}"
    done
    printf '%s\n' "${selected[@]}"

# Build images for selected Compose profiles without starting containers.
[group('Compose')]
compose-build *profiles:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t selected < <(just _compose-selected-profiles {{profiles}})
    flags=()
    for profile in "${selected[@]}"; do flags+=(--profile "$profile"); done
    build_args=(--build-arg HTTP_PROXY --build-arg HTTPS_PROXY --build-arg ALL_PROXY --build-arg NO_PROXY)
    docker compose "${flags[@]}" build "${build_args[@]}"

# Start selected Compose profiles; no arguments starts all profiles.
[group('Compose')]
compose-up *profiles:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t selected < <(just _compose-selected-profiles {{profiles}})
    flags=()
    for profile in "${selected[@]}"; do flags+=(--profile "$profile"); done
    build_args=(--build-arg HTTP_PROXY --build-arg HTTPS_PROXY --build-arg ALL_PROXY --build-arg NO_PROXY)
    docker compose "${flags[@]}" build "${build_args[@]}"
    docker compose "${flags[@]}" up -d
    if printf '%s\n' "${selected[@]}" | grep -qx mahfel; then just trust-local-ca; fi

# Stop the local Compose stack. Named volumes are retained.
[group('Compose')]
compose-down:
    docker compose down

# Stop every Compose profile and remove all named volumes for a fresh local start.
[group('Compose')]
compose-clean:
    docker compose --profile mahfel --profile infrastructure --profile observability down --volumes --remove-orphans

# Follow logs from selected Compose profiles; no arguments means all profiles.
[group('Compose')]
compose-logs *profiles:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t selected < <(just _compose-selected-profiles {{profiles}})
    flags=()
    for profile in "${selected[@]}"; do flags+=(--profile "$profile"); done
    docker compose "${flags[@]}" logs -f

# Install and trust Caddy's generated local HTTPS root certificate.
[group('Local development')]
trust-local-ca:
    #!/usr/bin/env bash
    set -euo pipefail
    certificate_dir=".local/caddy"
    certificate_path="${certificate_dir}/root.crt"
    incoming_path="${certificate_path}.incoming"
    digest_path="${certificate_dir}/root.sha256"
    mkdir -p "${certificate_dir}"
    trap 'rm -f "${incoming_path}"' EXIT

    file_digest() {
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum "$1" | awk '{print $1}'
        elif command -v shasum >/dev/null 2>&1; then
            shasum -a 256 "$1" | awk '{print $1}'
        else
            echo "No SHA-256 tool found (need sha256sum or shasum)" >&2
            return 1
        fi
    }

    linux_anchor_path() {
        if command -v update-ca-trust >/dev/null 2>&1; then
            printf '%s' "/etc/pki/ca-trust/source/anchors/mahfel-caddy-root.crt"
        elif command -v update-ca-certificates >/dev/null 2>&1; then
            printf '%s' "/usr/local/share/ca-certificates/mahfel-caddy-root.crt"
        fi
    }

    for attempt in {1..30}; do
        if docker compose cp caddy:/data/caddy/pki/authorities/local/root.crt "${incoming_path}"; then
            break
        fi
        if [[ "${attempt}" == 30 ]]; then
            echo "Caddy local root CA was not ready after 30 seconds" >&2
            exit 1
        fi
        sleep 1
    done
    # Keep the existing inode so running containers retain their bind mount.
    cat "${incoming_path}" > "${certificate_path}"
    chmod 0644 "${certificate_path}"
    incoming_digest="$(file_digest "${incoming_path}")"

    case "$(uname -s)" in
        Linux)
            anchor_path="$(linux_anchor_path)"
            if [[ -z "${anchor_path}" ]]; then
                echo "Unsupported Linux trust store; install ${certificate_path} manually" >&2
                exit 1
            fi
            if [[ -f "${anchor_path}" ]] && [[ "$(file_digest "${anchor_path}")" == "${incoming_digest}" ]]; then
                printf '%s' "${incoming_digest}" > "${digest_path}"
                echo "Caddy local root CA is already trusted; skipping trust update"
                exit 0
            fi
            ;;
        Darwin)
            if [[ -f "${digest_path}" ]] && [[ "$(<"${digest_path}")" == "${incoming_digest}" ]]; then
                echo "Caddy local root CA is already trusted; skipping trust update"
                exit 0
            fi
            ;;
        *)
            echo "Unsupported host OS; install ${certificate_path} manually" >&2
            exit 1
            ;;
    esac

    case "$(uname -s)" in
        Linux)
            sudo install -m 0644 "${certificate_path}" "${anchor_path}"
            if command -v update-ca-trust >/dev/null 2>&1; then
                sudo update-ca-trust
            else
                sudo update-ca-certificates
            fi
            ;;
        Darwin)
            sudo security add-trusted-cert -d -r trustRoot \
                -k /Library/Keychains/System.keychain "${certificate_path}"
            ;;
    esac
    printf '%s' "${incoming_digest}" > "${digest_path}"
    echo "Trusted Mahfel local CA. Restart open browsers before using https://mahfel.localhost"
