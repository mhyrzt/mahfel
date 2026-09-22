#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest_dir="${repo_dir}/deploy/minikube"
infra_dir="${manifest_dir}/infrastructure"
apps_dir="${manifest_dir}/applications"
namespace=mahfel

step_color=''
success_color=''
stdout_reset=''
error_color=''
stderr_reset=''
if [[ -z "${NO_COLOR:-}" && "${TERM:-}" != dumb ]]; then
  if [[ -t 1 ]]; then
    step_color=$'\033[1;36m'
    success_color=$'\033[1;32m'
    stdout_reset=$'\033[0m'
  fi
  if [[ -t 2 ]]; then
    error_color=$'\033[1;31m'
    stderr_reset=$'\033[0m'
  fi
fi

step() {
  printf '\n%s==>%s %s\n' "${step_color}" "${stdout_reset}" "$1"
}

success() {
  printf '    %sDone%s: %s\n' "${success_color}" "${stdout_reset}" "$1"
}

error() {
  printf '    %sError%s: %s\n' "${error_color}" "${stderr_reset}" "$1" >&2
}

usage() {
  echo "Usage: $0 {build|up|forward|cert|status|down}" >&2
  exit 2
}

require_minikube() {
  [[ "$(kubectl config current-context)" == minikube ]] || {
    error 'Current kubectl context must be minikube'
    exit 1
  }
  minikube status >/dev/null || {
    error 'Minikube is not running; run just minikube-init first'
    exit 1
  }
}

show_failure_details() {
  local resource="$1" selector="$2"
  error "${resource} did not become ready. Current pods and recent events:"
  kubectl -n "${namespace}" get pods -l "${selector}" -o wide >&2 || true
  kubectl -n "${namespace}" describe pods -l "${selector}" |
    sed -n '/^Events:/,$p' >&2 || true
  printf '    Recent container logs (if available):\n' >&2
  kubectl -n "${namespace}" logs "${resource}" --all-containers=true --tail=40 --prefix=true >&2 || true
}

wait_for_rollout() {
  local name="$1" timeout="$2"
  step "Waiting for ${name} (up to ${timeout}; first image pulls can be slow)"
  if ! kubectl -n "${namespace}" rollout status "deployment/${name}" --timeout="${timeout}"; then
    show_failure_details "deployment/${name}" "app=${name}"
    return 1
  fi
  success "${name} is ready"
}

wait_for_job() {
  local name="$1" timeout="$2"
  step "Waiting for ${name} job (up to ${timeout})"
  if ! kubectl -n "${namespace}" wait --for=condition=complete "job/${name}" --timeout="${timeout}"; then
    show_failure_details "job/${name}" "batch.kubernetes.io/job-name=${name}"
    return 1
  fi
  success "${name} completed"
}

build() {
  require_minikube
  cd "${repo_dir}"
  for target in api media worker; do
    step "Building and loading mahfel-${target}"
    docker build -f backend/Dockerfile --target "${target}" -t "mahfel-${target}:local" .
    minikube image load "mahfel-${target}:local"
    success "mahfel-${target} loaded"
  done
  step 'Building and loading mahfel-web'
  docker build -f frontend/Dockerfile \
    --build-arg NEXT_PUBLIC_MAHFEL_API_URL=https://api.mahfel.localhost \
    -t mahfel-web:local .
  minikube image load mahfel-web:local
  success 'mahfel-web loaded'
}

trust_host_ca() {
  local certificate_path="${repo_dir}/.local/minikube/root.crt"
  case "$(uname -s)" in
    Linux)
      if command -v update-ca-trust >/dev/null 2>&1; then
        sudo install -m 0644 "${certificate_path}" /etc/pki/ca-trust/source/anchors/mahfel-minikube-caddy-root.crt || return 1
        sudo update-ca-trust || return 1
      elif command -v update-ca-certificates >/dev/null 2>&1; then
        sudo install -m 0644 "${certificate_path}" /usr/local/share/ca-certificates/mahfel-minikube-caddy-root.crt || return 1
        sudo update-ca-certificates || return 1
      else
        echo "Unsupported Linux trust store; install ${certificate_path} manually" >&2
        return 1
      fi
      ;;
    Darwin)
      sudo security add-trusted-cert -d -r trustRoot \
        -k /Library/Keychains/System.keychain "${certificate_path}" || return 1
      ;;
    *)
      echo "Unsupported host OS; install ${certificate_path} manually" >&2
      return 1
      ;;
  esac
  echo 'Trusted Minikube Caddy CA. Restart open browsers before using https://mahfel.localhost'
}

export_and_trust_ca() {
  require_minikube
  mkdir -p "${repo_dir}/.local/minikube"
  kubectl -n "${namespace}" exec deployment/caddy -- \
    cat /data/caddy/pki/authorities/local/root.crt > "${repo_dir}/.local/minikube/root.crt"
  echo "Exported ${repo_dir}/.local/minikube/root.crt"
  trust_host_ca || {
    error 'Host CA installation failed. The cluster is ready; run just minikube-cert in an interactive terminal to finish HTTPS trust.'
    return 1
  }
}

config_map() {
  local name="$1" file="$2"
  kubectl -n "${namespace}" create configmap "${name}" \
    --from-file="${file}" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
}

up() {
  step 'Checking Minikube'
  require_minikube
  local node_ip
  node_ip="$(minikube ip)"
  [[ "${node_ip}" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo 'minikube ip did not return an IPv4 address' >&2
    exit 1
  }

  step 'Creating namespace and local configuration'
  kubectl apply -f "${infra_dir}/namespace.yaml" >/dev/null
  config_map keycloak-realm "${repo_dir}/deploy/keycloak/rastai-realm.json"
  config_map caddy-config "${repo_dir}/deploy/caddy/Caddyfile"
  config_map caddy-index "${repo_dir}/deploy/caddy/index.html"
  step 'Applying infrastructure (image pulls may take several minutes)'
  kubectl -n "${namespace}" apply \
    -f "${infra_dir}/credentials.yaml" \
    -f "${infra_dir}/postgres.yaml" \
    -f "${infra_dir}/redis.yaml" \
    -f "${infra_dir}/rabbitmq.yaml" \
    -f "${infra_dir}/rustfs.yaml" \
    -f "${infra_dir}/keycloak.yaml" \
    -f "${infra_dir}/caddy.yaml" >/dev/null
  sed "s/192\.0\.2\.1/${node_ip}/g" "${infra_dir}/coturn.yaml" |
    kubectl -n "${namespace}" apply -f - >/dev/null

  wait_for_rollout caddy 15m
  wait_for_rollout rabbitmq 5m
  step 'Preparing Caddy local CA for application pods'
  local caddy_pod
  caddy_pod="$(kubectl -n "${namespace}" get pod -l app=caddy -o jsonpath='{.items[0].metadata.name}')"
  kubectl -n "${namespace}" exec "${caddy_pod}" -- \
    wget --no-check-certificate -qO- https://caddy.mahfel.localhost >/dev/null || true
  for _ in {1..60}; do
    if kubectl -n "${namespace}" exec "${caddy_pod}" -- \
      test -s /data/caddy/pki/authorities/local/root.crt; then break; fi
    sleep 2
  done
  kubectl -n "${namespace}" exec "${caddy_pod}" -- \
    test -s /data/caddy/pki/authorities/local/root.crt || {
      error 'Caddy did not create its local CA within two minutes'
      show_failure_details deployment/caddy app=caddy
      return 1
    }

  local ca_file
  ca_file="$(mktemp)"
  trap 'rm -f "${ca_file}"' EXIT
  kubectl -n "${namespace}" exec "${caddy_pod}" -- \
    cat /data/caddy/pki/authorities/local/root.crt > "${ca_file}"
  kubectl -n "${namespace}" create configmap local-ca \
    --from-file="root.crt=${ca_file}" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
  rm -f "${ca_file}"
  trap - EXIT
  success 'Caddy local CA is available to application pods'

  step 'Running database seed and storage initialization'
  kubectl -n "${namespace}" apply -f "${apps_dir}/seed.yaml" >/dev/null
  kubectl -n "${namespace}" apply -f "${infra_dir}/rustfs-init.yaml" >/dev/null
  wait_for_job mahfel-seed 5m
  wait_for_job rustfs-init 15m
  step 'Applying application deployments'
  for manifest in api web media; do
    sed "s/192\.0\.2\.1/${node_ip}/g" "${apps_dir}/${manifest}.yaml" |
      kubectl -n "${namespace}" apply -f - >/dev/null
  done
  kubectl -n "${namespace}" apply -f "${apps_dir}/worker.yaml" >/dev/null
  wait_for_rollout mahfel-api 5m
  wait_for_rollout mahfel-web 5m
  wait_for_rollout mahfel-media 5m
  wait_for_rollout mahfel-worker 5m
  step 'Installing local HTTPS certificate'
  export_and_trust_ca
  success 'Minikube stack is ready'
  echo 'Next: run just minikube-forward in another terminal, then open https://mahfel.localhost.'
}

case "${1:-}" in
  build) build ;;
  up) up ;;
  forward)
    require_minikube
    step 'Forwarding local HTTP and HTTPS to Minikube Caddy (sudo required for ports 80 and 443)'
    kubeconfig_path="${KUBECONFIG:-${HOME}/.kube/config}"
    kubectl_bin="$(command -v kubectl)"
    sudo env KUBECONFIG="${kubeconfig_path}" "${kubectl_bin}" -n "${namespace}" \
      port-forward --address 127.0.0.1 service/caddy 80:80 443:443 || {
        error 'Port forwarding failed. Check sudo/kubeconfig access and make sure host ports 80 and 443 are free.'
        exit 1
      }
    ;;
  cert)
    export_and_trust_ca
    ;;
  status)
    require_minikube
    kubectl -n "${namespace}" get pods,services,jobs,pvc
    ;;
  down)
    require_minikube
    kubectl delete namespace "${namespace}" --ignore-not-found
    ;;
  *) usage ;;
esac
