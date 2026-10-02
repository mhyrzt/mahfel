# Kubernetes deployments

Portable Kubernetes resources with explicit environment profiles:

- `base/`: four application deployments/services, private operations ports, probes and resource budgets.
- `overlays/production/`: single-VPS production on K3s, persistent PostgreSQL/RabbitMQ, ephemeral Redis and host-network Coturn; external
  Keycloak and object storage. Two mediasoup workers expose TCP/UDP `40000–40001`.
- `overlays/local/`: isolated development fixtures and Minikube lifecycle, moved from
  `deploy/minikube/`. Includes local identity/storage/TURN, seeds and monitoring; never apply to initial production.
- `overlays/production/secrets/`: encrypted SealedSecrets populated after cluster bootstrap.
- `examples/`: secret templates with placeholders only; populated copies belong in ignored `.local/`.

Start with the [initial production runbook](RUNBOOK.md). Argo CD resources live in [deploy/argocd](../argocd/README.md).
Application containers remain stateless; infrastructure PVCs are node-local and require external backups.

```bash
just kubernetes render production
just kubernetes lint production
just kubernetes lint local
just kubernetes local init
just kubernetes local bootstrap
```

`just minikube ...` remains a convenience alias for the local profile. No generic recipe automatically
applies initial production manifests or destroys a namespace. Argo CD owns ordinary production rollout.

Production values are preparation placeholders, not a ready-to-launch configuration. Supply actual endpoints,
public IP, secrets, TLS, storage and pullable image digests before syncing. Offline schema checks do
not prove networking, image availability, login, recording or load capacity.

See [configuration and secret management](CONFIGURATION.md) for Sealed Secrets, GitOps rollout and key recovery.
