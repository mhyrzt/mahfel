# Kubernetes production deployment

The initial production target is a dedicated VPS running K3s with Argo CD. External Keycloak and S3-compatible object storage remain outside this VPS. Coturn at
`turn.rastai.ai` is now deployed on the same Kubernetes node.

See the repository [production runbook](https://hamgit.ir/hamdars/mahfel/-/blob/main/deploy/kubernetes/RUNBOOK.md)
for provisioning, secrets, DNS/TLS, migrations, image selection, backups and acceptance.
See [Argo CD preparation](https://hamgit.ir/hamdars/mahfel/-/blob/main/deploy/argocd/README.md)
for private bootstrap access and controlled GitOps synchronization.

## Layout and operating model

- `deploy/kubernetes/base/`: four application deployments/services.
- `deploy/kubernetes/overlays/production/`: single-node resources, two RTC workers on TCP/UDP
  `40000–40001`, resource limits, persistent PostgreSQL/RabbitMQ and ephemeral Redis.
- `deploy/kubernetes/overlays/local/`: moved Minikube fixtures, local credentials, seeds and monitoring.
- `deploy/k3s/`: K3s configuration example.
- `deploy/argocd/`: restricted AppProject, manually synced Application and exact-version bootstrap.

```bash
just kubernetes render production
just kubernetes lint production
just kubernetes lint local
```

Configuration contains deliberate placeholders. A valid rendered manifest is not ready for production
until endpoints, public IP, TLS, secrets and verified pullable image digests are supplied.
CI builds/publishes only; publishing a tag does not deploy it. Argo CD owns normal deployment after
reviewed image/configuration changes reach Git. Migration hooks require full synchronization.

Single-node local volumes and media host ports are explicit initial deployment constraints. Infrastructure
PVCs are protected from ordinary Argo pruning, but external backups and restore tests remain required.
Applications remain stateless except volatile media/session resources and temporary recording/export
files. See [ADR 0031](../adr/0031-vps-kubernetes-production.md) for the hosting decision and [ADR 0032](../adr/0032-cluster-turn-and-sealed-secrets.md) for in-cluster TURN/secrets.

## Validation boundary

Offline Kustomize/schema checks validate manifest structure. Server-side dry runs, image pulls,
probe readiness, TLS, login, private networking, direct WebRTC, forced TURN, recording/export and
resource/load tests must be performed on the actual VPS before opening service access.
