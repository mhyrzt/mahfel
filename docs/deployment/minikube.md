# Minikube development stack

This stack runs the four Mahfel application containers and local PostgreSQL, Redis, RabbitMQ,
RustFS, Keycloak, Caddy, and coturn in a **single-node, local-only** Minikube cluster. It uses the
same development fixture credentials and realm as Compose. Keep the `mahfel` namespace on a private
Minikube profile; the committed credentials are not suitable for shared clusters.

## Start

Install Docker, Minikube, and kubectl. Allocate enough resources for the three Rust builds,
Keycloak, and the application (at least 4 CPUs and 8 GiB of memory is a useful starting point).
From the repository root:

```bash
just minikube bootstrap
```

`just minikube bootstrap` initializes the Docker-driver cluster and kubectl context when needed,
builds and deploys the apps, and keeps HTTPS forwarding open until interrupted. Host ports 80 and 443
must be free. Run `just minikube status` in another terminal to inspect the stack.

The first media image build is slow because mediasoup builds native components. Container builds
need access to `static.rust-lang.org` to install the pinned Rust components. `build` builds
the API, media, worker, and web Docker targets using the host network and configured proxy
environment, then loads the resulting `:local` images into
Minikube. Re-run `build` after source changes and `up` to apply configuration. To restart an
unchanged-tag image after rebuilding, use `kubectl -n mahfel rollout restart deployment/mahfel-api`
(or the corresponding deployment). The API image includes `backend/seeds/`, including assets kept
inside that directory. The seed Job runs after migrations and RustFS bucket initialization; every
`up` recreates it so the image's current TOML fixtures are applied. Rebuild the API image with
`just minikube build` before `up` when seed files change. File paths in a fixture are relative to
that TOML file, or absolute within the seed container.

Run `just minikube fmt` to format Minikube YAML manifests, `just minikube fmt-check` to check
formatting, and `just minikube lint` to render and validate the complete Kubernetes resource set.

The helper checks that the current kubectl context is exactly `minikube` before changing resources.
It creates ConfigMaps from the repository's canonical Caddyfile, index page, and Keycloak realm
fixture; no copies of these files are maintained in the Minikube directory. Manifests are in
`deploy/kubernetes/overlays/local/infrastructure/` and `deploy/kubernetes/overlays/local/applications/`, with a file per component.
The root Kustomization renders them with `kubectl kustomize deploy/kubernetes/overlays/local` for inspection. The helper
applies them in dependency order so migrations and seeds complete before the API starts. The
documentation-only address `192.0.2.1` in the checked-in manifests is replaced with the actual
`minikube ip` during `up` for RTC traffic. The OIDC host alias uses `192.0.2.2` as a placeholder
and is replaced with the in-cluster Caddy Service IP. The web pod uses this for Keycloak requests;
the API's OIDC HTTP client uses the same IP as a DNS override for discovery and JWKS requests while
preserving the public issuer hostname and TLS certificate. Applying the Kustomization directly will
not configure RTC or OIDC networking correctly.

## Browser HTTPS

`just minikube bootstrap` starts forwarding automatically. When running the steps individually,
keep this running in a second terminal:

```bash
just minikube forward
```

The forward prompts for `sudo` because ports 80 and 443 are privileged. It uses your kubectl
configuration and binds only `127.0.0.1:80` and `127.0.0.1:443`. These ports must be free. The
existing `*.mahfel.localhost` names resolve to loopback on normal development hosts. `just minikube up`
exports and trusts the Minikube Caddy CA
on the host. If it could not prompt for `sudo`, or the CA changes, run this in an interactive terminal:

```bash
just minikube cert
```

The CA is saved at `.local/minikube/root.crt`. The Minikube Caddy has its own CA; the Compose Caddy
certificate does not trust it. The API and web pods mount the Minikube CA to reach Keycloak at its
issuer URL.

Some browsers use a separate trust store. Follow that browser's certificate import workflow if
HTTPS still reports an untrusted issuer.

Check the browser-facing routes and pod readiness:

```bash
curl https://api.mahfel.localhost/v1/status
curl https://mahfel.localhost/api/health
kubectl -n mahfel get pods
kubectl -n mahfel logs job/mahfel-seed
```

S3 is at `https://s3.mahfel.localhost`; the RustFS console is at
`https://storage.mahfel.localhost`. RabbitMQ management is at
`https://rabbitmq.mahfel.localhost`. Grafana is at `https://grafana.mahfel.localhost` and includes
Prometheus, Loki, and Tempo data sources. Prometheus scrapes the API, media, and worker operations
endpoints plus kube-state-metrics and node-exporter; its data is retained for seven days. Internal
gRPC, operations, and Prometheus ports remain ClusterIP only. Node-exporter reports the Minikube
node environment (a Docker container when using the Docker driver), not the physical host's metrics.

## Media tests

The media pod publishes only its RTC ports through `hostPort`; coturn uses the Minikube node
network for its relay range. The helper sets the media announced
address and coturn external address to `minikube ip`. RTC uses ports 40000–40003 UDP/TCP and
TURN uses 3478 UDP/TCP plus 49160–49200 UDP. The browser host must be able to reach the
Minikube node IP and these ports. This works most directly on Linux with the Docker driver.
On macOS, Windows, or WSL, Docker-driver node networking may prevent direct UDP access; HTTP
and API tests still work, but a successful browser media test needs a VM driver or another route
to the node. `kubectl port-forward` cannot carry UDP. Do not treat a passing HTTP health check as
proof of WebRTC connectivity.

## Stop and inspect

```bash
kubectl -n mahfel describe pod <pod-name>
kubectl -n mahfel logs deployment/mahfel-api
just minikube down
just minikube stop
# Full profile reset, including Minikube node state and node-resident images:
just minikube clean-aggressive
```

`just minikube down` deletes the `mahfel` namespace and its local PVCs; `just minikube stop` stops the
cluster while retaining its profile and node-resident images. `just minikube clean-aggressive` deletes
the `mahfel` namespace and its PVC-backed local data, then deletes only the default `minikube` profile;
this removes its node state and loaded images but does not delete other Minikube profiles or host
Docker images/build cache. Nothing in this directory is a production Kubernetes deployment: it
uses single replicas, in-cluster fixture credentials, a local CA, and host networking for coturn's
relay ports.
