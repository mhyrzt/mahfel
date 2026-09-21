#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest_dir="${repo_dir}/deploy/minikube"
infra_dir="${manifest_dir}/infrastructure"
apps_dir="${manifest_dir}/applications"
namespace=mahfel

usage() {
  echo "Usage: $0 {build|up|forward|cert|status|down}" >&2
  exit 2
}

require_minikube() {
  [[ "$(kubectl config current-context)" == minikube ]] || {
    echo 'Current kubectl context must be minikube' >&2
    exit 1
  }
  minikube status >/dev/null
}

build() {
  require_minikube
  cd "${repo_dir}"
  for target in api media worker; do
    docker build -f backend/Dockerfile --target "${target}" -t "mahfel-${target}:local" .
    minikube image load "mahfel-${target}:local"
  done
  docker build -f frontend/Dockerfile \
    --build-arg NEXT_PUBLIC_MAHFEL_API_URL=https://api.mahfel.localhost \
    -t mahfel-web:local .
  minikube image load mahfel-web:local
}

config_map() {
  local name="$1" file="$2"
  kubectl -n "${namespace}" create configmap "${name}" \
    --from-file="${file}" --dry-run=client -o yaml | kubectl apply -f -
}

up() {
  require_minikube
  local node_ip
  node_ip="$(minikube ip)"
  [[ "${node_ip}" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo 'minikube ip did not return an IPv4 address' >&2
    exit 1
  }

  kubectl apply -f "${infra_dir}/namespace.yaml"
  config_map keycloak-realm "${repo_dir}/deploy/keycloak/rastai-realm.json"
  config_map caddy-config "${repo_dir}/deploy/caddy/Caddyfile"
  config_map caddy-index "${repo_dir}/deploy/caddy/index.html"
  kubectl -n "${namespace}" apply \
    -f "${infra_dir}/credentials.yaml" \
    -f "${infra_dir}/postgres.yaml" \
    -f "${infra_dir}/redis.yaml" \
    -f "${infra_dir}/rabbitmq.yaml" \
    -f "${infra_dir}/rustfs.yaml" \
    -f "${infra_dir}/keycloak.yaml" \
    -f "${infra_dir}/caddy.yaml"
  sed "s/192\.0\.2\.1/${node_ip}/g" "${infra_dir}/coturn.yaml" |
    kubectl -n "${namespace}" apply -f -

  kubectl -n "${namespace}" rollout status deployment/caddy --timeout=180s
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
    test -s /data/caddy/pki/authorities/local/root.crt

  local ca_file
  ca_file="$(mktemp)"
  trap 'rm -f "${ca_file}"' EXIT
  kubectl -n "${namespace}" exec "${caddy_pod}" -- \
    cat /data/caddy/pki/authorities/local/root.crt > "${ca_file}"
  kubectl -n "${namespace}" create configmap local-ca \
    --from-file="root.crt=${ca_file}" --dry-run=client -o yaml | kubectl apply -f -
  rm -f "${ca_file}"
  trap - EXIT

  kubectl -n "${namespace}" apply -f "${apps_dir}/seed.yaml"
  kubectl -n "${namespace}" apply -f "${infra_dir}/rustfs-init.yaml"
  kubectl -n "${namespace}" wait --for=condition=complete job/mahfel-seed --timeout=300s || {
    kubectl -n "${namespace}" logs job/mahfel-seed --all-containers=true
    exit 1
  }
  kubectl -n "${namespace}" wait --for=condition=complete job/rustfs-init --timeout=300s || {
    kubectl -n "${namespace}" logs job/rustfs-init
    exit 1
  }
  for manifest in api web media; do
    sed "s/192\.0\.2\.1/${node_ip}/g" "${apps_dir}/${manifest}.yaml" |
      kubectl -n "${namespace}" apply -f -
  done
  kubectl -n "${namespace}" apply -f "${apps_dir}/worker.yaml"
  kubectl -n "${namespace}" rollout status deployment/mahfel-api --timeout=180s
  kubectl -n "${namespace}" rollout status deployment/mahfel-web --timeout=180s
  echo 'Run minikube.sh forward in another terminal, then minikube.sh cert to export the local CA.'
}

case "${1:-}" in
  build) build ;;
  up) up ;;
  forward)
    require_minikube
    kubectl -n "${namespace}" port-forward --address 127.0.0.1 service/caddy 80:80 443:443
    ;;
  cert)
    require_minikube
    mkdir -p "${repo_dir}/.local/minikube"
    kubectl -n "${namespace}" exec deployment/caddy -- \
      cat /data/caddy/pki/authorities/local/root.crt > "${repo_dir}/.local/minikube/root.crt"
    echo "Exported ${repo_dir}/.local/minikube/root.crt"
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
