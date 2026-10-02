# Mahfel initial production: single VPS, K3s and Argo CD

This is deployment preparation, not evidence of a deployed or load-tested service. Record the actual versions, image digests and acceptance results here
or in the operator's deployment record.

## 1. Placement and sizing

| Location | Services |
| --- | --- |
| VPS | K3s, Argo CD, Sealed Secrets, four Mahfel apps, PostgreSQL, Redis, RabbitMQ, Traefik, Coturn |
| External | Existing Keycloak, S3-compatible object storage |

Initial estimate for light usage: **4 vCPU / 8 GB RAM / 80 GB SSD or NVMe**, preferably x86-64 to
match the published images. This is not a participant-capacity promise. One media replica, two
mediasoup workers and one export at a time are configured. The export worker is capped at one CPU.
Reserve 25–30 GB free for temporary files; database/queue volumes request 15/5 GiB. Check real disk
usage: local-path PVC requests and emptyDir size limits do not preallocate disk or provide backups.
Avoid building production images on this VPS. Heavy monitoring is opt-in through the local profile,
not part of initial production; collect basic pod/resource metrics and logs first.

- [ ] Record VPS public IPv4, provider firewall, bandwidth/traffic allowance and disk capacity.
- [ ] Ensure the provider permits inbound RTC UDP/TCP without port remapping.
- [ ] Configure SSH administration, OS updates, time synchronization and off-server backups.
- [ ] Accept the single-node failure boundary: VPS loss stops calls and all local services.

## 2. Install K3s

For the fresh VPS, use **Ubuntu 24.04 LTS (x86-64)**. The
[VPS bootstrap script](../vps/README.md) automates host prerequisites, firewall, pinned K3s,
private Argo CD and Sealed Secrets setup. Start with its `--plan` command, then run `--apply`
on the intended VPS. It leaves application deployment pending production configuration.
After using it, continue at section 3; K3s, Argo CD and the sealing controller are already installed.

For manual setup, use an exact reviewed K3s version. Check the corresponding
Kubernetes version against Argo CD's supported versions. Review the upstream install script before
executing it; the commands below run on the VPS, not on a developer's current cluster.

```bash
sudo mkdir -p /etc/rancher/k3s
sudo install -m 0600 deploy/k3s/config.yaml.example /etc/rancher/k3s/config.yaml
sudoedit /etc/rancher/k3s/config.yaml
# Replace documentation IPs with the actual public address first.
curl -fsSL https://get.k3s.io -o /tmp/install-k3s.sh
# Review /tmp/install-k3s.sh, then choose an exact supported release:
sudo env INSTALL_K3S_VERSION='<exact-k3s-release>' sh /tmp/install-k3s.sh
sudo k3s kubectl get nodes
```

Set up a private administrator kubeconfig securely; do not make `/etc/rancher/k3s/k3s.yaml` world-readable.
For commands below, `kubectl` must target this VPS cluster. Verify `kubectl config current-context`
before every apply. Keep default Traefik, ServiceLB, local-path storage and network-policy enforcement.
The supplied HelmChartConfig enables HTTP-to-HTTPS redirect; verify chart reconciliation and the
redirect on your selected K3s release.

```bash
kubectl get storageclass local-path
kubectl -n kube-system get pods -l app.kubernetes.io/name=traefik
kubectl apply -f deploy/k3s/traefik.yaml
kubectl label node mahfel-production mahfel.io/media=true
```

Use the actual node name if different. The media label pins the single media replica to the node
whose public address is configured. Do not scale media replicas on this node: fixed host ports conflict.
For another Kubernetes distribution, qualify ingress, network-policy enforcement, storage class and
hostPort support, then patch these settings rather than assuming K3s defaults exist.

## 3. DNS, HTTPS and firewall

- [ ] Point `mahfel.rastai.ai` and `mahfel-api.rastai.ai` to the VPS inbound IP.
- [ ] Open TCP 80/443 for Traefik; default K3s ServiceLB occupies these host ports.
- [ ] Open UDP **and TCP 40000–40001** for the configured media workers. Match the hostPort,
      container port, firewall and announced IP. RTC traffic bypasses HTTP ingress.
- [ ] Keep SSH administration restricted; Kubernetes API 6443 is private/VPN/SSH-admin access only.
- [ ] Keep PostgreSQL 5432, Redis 6379, RabbitMQ 5672/15672, gRPC 50051, registration 50052 and
      operations 9101–9103 private. Never expose Flannel VXLAN 8472 publicly.
- [ ] Point `turn.rastai.ai` to this VPS; the former Coturn VPS has been removed.
- [ ] Open TCP/UDP 3478 and UDP relay ports 49160–49359 on the VPS/provider firewall.
      Keep 5349 closed until TURN TLS is configured and tested.
- [ ] Verify in-cluster Coturn can reach the announced media address on 40000–40001.
      Same-node public-IP/NAT routing must be tested, not assumed.

Prepare a valid certificate covering both web/API names and an automated renewal process. Its
certificate and private key are sealed as `mahfel-ingress-tls` in the workflow below. A certificate
controller is not bundled yet; renewal must reseal/sync the updated certificate before expiry.

```bash
kubectl apply -f deploy/kubernetes/overlays/production/namespace.yaml
```

Ingress includes TLS routing; verify HTTP redirects to HTTPS with the installed Traefik configuration
before admitting users. NetworkPolicy permits Traefik's standard kube-system labels; verify actual
controller labels and client connectivity. Policies do not replace the VPS/provider firewall.

## 4. External dependencies

### Keycloak

- [ ] Supply actual public issuer and server base URL/realm in `backend.env` and `web.env`.
- [ ] Configure confidential `mahfel-web` with PKCE and exact callback
      `https://mahfel.rastai.ai/api/auth/callback/keycloak`.
- [ ] Configure the `mahfel-api` audience, required basic claims, and `app_user_id` mapper.
- [ ] Configure the `mahfel-admin` service account and minimum provisioning permissions.
- [ ] Verify web and API can reach discovery, JWKS and administration endpoints using trusted TLS.
- [ ] Establish the initial production administrator through the identity/bootstrap procedure;
      never apply the development seed Job or import the local fixture credentials.

See [Keycloak configuration](../keycloak/README.md). External hosting does not remove Mahfel-specific
clients, audience/mappers and service-account requirements.

### Object storage

- [ ] Create one private bucket and scoped read/write/delete/multipart credentials.
- [ ] Supply the internal/service-reachable and browser-reachable HTTPS endpoints, bucket, region
      and provider's path-style requirement in `backend.env`.
- [ ] Set `MAHFEL_RENDER_ASSET_ORIGINS` to the exact origin of the endpoint used in archived asset URLs.
- [ ] Configure CORS for `https://mahfel.rastai.ai`, GET/PUT, needed request headers and exposed ETag.
- [ ] Clean abandoned multipart/staging uploads; coordinate accepted recording deletion with Mahfel.
- [ ] Verify a browser presigned upload/download and server-side read/write/delete.

### Coturn

Coturn is an Argo-managed single Deployment with host networking, pinned to the same media node.
It directly binds 3478 and its configured relay range; there is no HTTP ingress or public Kubernetes
Service for TURN. Host-network traffic needs host/provider firewall rules: namespace NetworkPolicy
is not its access-control boundary. No durable TURN allocation storage is required.

Copy `examples/turnserver.conf.example` to `.local/kubernetes/turnserver.conf` and protect it with
mode 0600. Set `relay-ip` to an address assigned to the node, `external-ip` to the public IP (or
public/private mapping for NAT), and `allowed-peer-ip` to the actual announced media public IPv4.
The initial allowlist limits relaying to that media address; adding other media nodes requires
reviewed allowlist changes. Set `static-auth-secret` to the same value as API
`MAHFEL_TURN__STATIC_SECRET`, with matching realm `turn.rastai.ai`. No anonymous TURN is enabled.

Seal this file as `mahfel-coturn-config` using [the shared sealing workflow](CONFIGURATION.md).
Do not commit the populated plaintext configuration.

The API mounts `turn.toml` for the URI array. Initial listeners are UDP/TCP 3478; TLS/DTLS are
explicitly disabled. To add TURN TLS, provision a renewed certificate Secret for `turn.rastai.ai`,
mount it in Coturn, replace `no-tls` with certificate/key/TLS-port settings, open TCP 5349, and add
the verified `turns:` URI. This is a deployment change requiring configuration and connectivity
checks; mounting the web/API certificate alone does not configure Coturn TLS.

TCP socket readiness proves only listener availability. Before use, test authenticated allocation,
rejection of invalid credentials, forced TURN calls, allowed peer behavior and relay port capacity.
The 200-port relay range and quota are initial bounds, not a participant-capacity guarantee.
Coturn pod replacement interrupts its existing relayed calls; use a quiet rollout window.

## 5. Configuration, images and secrets

See [GitOps environment handling](CONFIGURATION.md) for the current configuration/secret split and rollout behavior.

Edit `deploy/kubernetes/overlays/production/backend.env`, `web.env`, `turn.toml` and `kustomization.yaml`.
Replace all `CHANGEME` entries and the documentation IP `192.0.2.1`. These files contain non-secret
settings only. Backend selects profile `production`, falling back to bundled `default.toml`; it intentionally
does not load the `docker.toml` development proxy setting. API additionally loads mounted `local.toml`
for TURN URLs. Add a trusted proxy only after verifying the actual source IP; otherwise forwarded
client-IP headers remain untrusted (client-IP rate limiting needs qualification behind ingress).

- [ ] Verify the four image references exist and can be pulled; successful builds/release entries
      do not prove publication. Registry publishing permissions were an outstanding release gate.
- [ ] Replace preparation tags with reviewed immutable digests in Kustomize `images` entries:
      use `digest: sha256:<verified-digest>` and remove `newTag`.
- [ ] Verify the web image was built with public API URL `https://mahfel-api.rastai.ai`;
      runtime environment changes cannot rewrite the compiled browser URL.
- [ ] Pin infrastructure images by digest after selecting reviewed compatible versions.

Copy templates into the ignored `.local/` directory and populate them through your secret tooling.
Use strong URL-safe Redis/RabbitMQ passwords, or percent-encode them in connection URLs. Keep
infrastructure password values consistent with backend database/Redis/AMQP credentials.

```bash
mkdir -p .local/kubernetes
chmod 700 .local/kubernetes
cp deploy/kubernetes/examples/infrastructure.env.example .local/kubernetes/infrastructure.env
cp deploy/kubernetes/examples/backend-secrets.env.example .local/kubernetes/backend-secrets.env
cp deploy/kubernetes/examples/web-secrets.env.example .local/kubernetes/web-secrets.env
chmod 600 .local/kubernetes/*.env
# Populate these files privately; do not execute the next commands with CHANGEME values.
```

Derive the media token as base64(HMAC-SHA256(registration-root-secret, `media-production-1`)); node ID and
root secret must match the configured API values. Use the same dedicated render key for web
`MAHFEL_RENDER_API_KEY` and worker `MAHFEL_WORKER__EXPORTS__RENDERER_API_KEY`.
Seal these credentials with the [Sealed Secrets workflow](CONFIGURATION.md). Bootstrap Argo CD and
its Sealed Secrets controller first, fetch the intended cluster's public certificate, and provide
`.local/kubernetes/registry.json`, `tls.crt`, `tls.key` and the populated Coturn configuration.

```bash
./deploy/kubernetes/seal-secrets.sh .local/kubernetes/sealed-secrets.pem
# Review and commit/push the encrypted output before normal GitOps rollout.
kubectl -n mahfel apply -k deploy/kubernetes/overlays/production/secrets
kubectl -n mahfel wait --for=condition=Synced sealedsecret --all --timeout=2m
```

Initial `secrets/kustomization.yaml` is intentionally empty until real credentials can be sealed.
Do not start workloads until all six Secret resources have unsealed successfully. The resulting
Kubernetes Secrets use stable names; coordinated pod replacement is still required after credential
rotation. PostgreSQL/RabbitMQ identities require explicit server-side password changes too.

## 6. Infrastructure preparation and validation

Render and validate without deploying, then validate against the actual cluster:

```bash
just kubernetes lint production
kubectl apply --dry-run=server -k deploy/kubernetes/overlays/production
```

Pre-create infrastructure for initial setup after provisioning all referenced Secrets (including Coturn),
so the RabbitMQ policy exists before jobs run:

```bash
kubectl -n mahfel apply -k deploy/kubernetes/overlays/production/infrastructure
kubectl -n mahfel rollout status deployment/postgres --timeout=5m
kubectl -n mahfel rollout status deployment/redis --timeout=5m
kubectl -n mahfel rollout status deployment/rabbitmq --timeout=5m
kubectl -n mahfel rollout status deployment/coturn --timeout=5m
kubectl -n mahfel exec deployment/rabbitmq -- rabbitmqctl set_policy -p mahfel mahfel-job-dlx \
  '^mahfel\.jobs\.(files\.cleanup|recordings\.(finalize|export))\.v1$' \
  '{"dead-letter-strategy":"at-least-once","overflow":"reject-publish","dead-letter-exchange":"mahfel.jobs.dead.v1","max-length-bytes":104857600}' \
  --apply-to quorum_queues
```

RabbitMQ uses a stable hostname/node name and persistent data, with one broker and a dedicated
`mahfel` vhost. This is not replicated quorum resilience. Argo CD subsequently adopts these same
resources. PostgreSQL's initial `mahfel` login is also the bootstrap/migration owner; restrict it to
this application/network and separate runtime/migration privileges before broader production use.

## 7. Application GitOps and first rollout

Follow the [Argo CD bootstrap](../argocd/README.md). Use an exact supported release, private admin
access, read-only Git credentials and reviewed deployment configuration committed/pushed to Git.
This runbook does not itself commit, publish images or deploy the VPS.

1. Create the AppProject/Application after secrets, TLS, infrastructure and policy are ready.
2. Review the Argo diff and full-sync `mahfel-production` manually, with hooks enabled.
3. Infrastructure must become healthy before wave 0 runs `mahfel-api migrate` using the API image.
4. The migration must finish before ordinary application rollout; no development seeds run.
5. API/web start, then media/worker, then ingress. Check registration and readiness.
6. Inspect failed migration logs/database state before retry; do not bypass the hook with selective sync.

```bash
kubectl -n mahfel get pods,services,pvc,jobs,ingress
kubectl -n mahfel logs deployment/mahfel-media --tail=100
kubectl -n mahfel logs deployment/mahfel-worker --tail=100
kubectl -n mahfel top pods
```

## 8. Acceptance before opening access

- [ ] HTTPS/certificate renewal and HTTP redirect; login, provisioning, access control and logout.
- [ ] REST and WebSocket signaling through the API, reconnect and room admission.
- [ ] Direct WebRTC audio/video and screen sharing from different external networks.
- [ ] Forced TURN fallback via `turn.rastai.ai`; record packet loss, RTT and media quality.
- [ ] Upload/download and collaborative workspace behavior.
- [ ] Recording start/stop, finalization, interactive replay, MP4 export and deletion.
- [ ] Private worker-to-renderer authentication and allowlisted storage asset retrieval.
- [ ] Export while calls are active; check CPU throttling, RAM, free disk and network throughput.
- [ ] Verify private management/operations ports are unreachable from the Internet.
- [ ] Test backup restore into an isolated environment before declaring durable-state readiness.

Health checks: web `/api/health`; API/media/worker operations `/health/live` and `/health/ready` on
9101/9102/9103. Probe private endpoints through an admin tunnel. Readiness failures are not proof that
restarting the process will repair dependencies.

## 9. Updates, backups and recovery

CI builds/publishes; update reviewed image digests in Git, then review/manual-sync Argo CD. Migrations are forward-only: do not blindly revert to an older image against an
incompatible schema. Back up before schema-changing rollout and plan restore/compatibility.

Before media configuration/image changes, stop admitting new sessions operationally and wait for
active rooms/recordings to finish. `Recreate` prevents fixed-port collisions but does not automate
room draining or make call-preserving rollout possible. Worker rollout waits up to 60 seconds;
long exports may be interrupted/retried. API/web also have no zero-downtime guarantee on this size.

Back up PostgreSQL daily and before migrations; encrypt/copy backups off-server and test restores:

```bash
kubectl -n mahfel exec deployment/postgres -- pg_dump -U mahfel -d mahfel -Fc \
  > .local/kubernetes/mahfel-backup.dump
```

Protect and back up all Sealed Secrets controller keys off-server (see [configuration recovery](CONFIGURATION.md)),
Kubernetes bootstrap credentials, K3s configuration/token/datastore according to the selected
K3s datastore, and RabbitMQ definitions/policy. Export definitions through a protected management
tunnel without committing credentials. PostgreSQL is business truth; Redis can be rebuilt, but
RabbitMQ data loss can strand pending jobs, so verify application recovery before discarding queues.
External Keycloak/storage backups and retention remain the external services' responsibility.

After node failure: restore/replace K3s, restore protected settings/secrets and PostgreSQL, recover
RabbitMQ or reconcile pending jobs, verify image pulls, then full-sync and repeat acceptance.
Never delete PVCs, their underlying local-path directories or the namespace as a routine rollback.

References: [K3s requirements](https://docs.k3s.io/installation/requirements),
[K3s networking](https://docs.k3s.io/networking/networking-services),
[recording operations](../../docs/deployment/recordings.md).
