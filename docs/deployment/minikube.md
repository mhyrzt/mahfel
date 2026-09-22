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
just minikube-init
just minikube-bootstrap
```

Initialize the cluster once. `just minikube-bootstrap` builds and deploys the apps and keeps HTTPS
forwarding open until interrupted. Host ports 80 and 443 must be free. Run `just minikube-status`
in another terminal to inspect the stack.

The first media image build is slow because mediasoup builds native components. `build` builds
the API, media, worker, and web Docker targets and loads the resulting `:local` images into
Minikube. Re-run `build` after source changes and `up` to apply configuration. To restart an
unchanged-tag image after rebuilding, use `kubectl -n mahfel rollout restart deployment/mahfel-api`
(or the corresponding deployment). The seed Job runs once per namespace and applies the local
development data after migrations; delete that Job before `up` if seed files change.

The helper checks that the current kubectl context is exactly `minikube` before changing resources.
It creates ConfigMaps from the repository's canonical Caddyfile, index page, and Keycloak realm
fixture; no copies of these files are maintained in the Minikube directory. Manifests are in
`deploy/minikube/infrastructure/` and `deploy/minikube/applications/`, with a file per component.
The root Kustomization renders them with `kubectl kustomize deploy/minikube` for inspection. The helper
applies them in dependency order so migrations and seeds complete before the API starts. The
documentation-only address `192.0.2.1` in the checked-in manifests is replaced with the actual
`minikube ip` during `up`; applying the Kustomization directly will not configure RTC or OIDC
networking correctly.

## Browser HTTPS

`just minikube-bootstrap` starts forwarding automatically. When running the steps individually,
keep this running in a second terminal:

```bash
just minikube-forward
```

The forward prompts for `sudo` because ports 80 and 443 are privileged. It uses your kubectl
configuration and binds only `127.0.0.1:80` and `127.0.0.1:443`. These ports must be free. The
existing `*.mahfel.localhost` names resolve to loopback on normal development hosts. `just minikube-up`
exports and trusts the Minikube Caddy CA
on the host. If it could not prompt for `sudo`, or the CA changes, run this in an interactive terminal:

```bash
just minikube-cert
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
`https://rabbitmq.mahfel.localhost`. Internal gRPC and operations ports remain ClusterIP only.
No observability stack is installed by this local profile.

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
just minikube-down
just minikube-stop
```

`minikube-down` deletes the `mahfel` namespace and its local PVCs; `minikube-stop` stops the
cluster while retaining its profile and cached images. Nothing in this directory is a production
Kubernetes deployment: it uses single replicas, in-cluster fixture credentials, a local CA,
and host networking for coturn's relay ports.
