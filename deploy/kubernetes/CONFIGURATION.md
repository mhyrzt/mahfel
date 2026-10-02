# Production configuration and GitOps

## What is stored where

| Data | Git source | Kubernetes result |
| --- | --- | --- |
| Public URLs, storage endpoints/bucket, realm, limits | `overlays/production/backend.env`, `web.env` | Kustomize-generated ConfigMaps injected with `envFrom` |
| TURN client URLs | `overlays/production/turn.toml` | Generated ConfigMap mounted into API |
| Passwords, client secrets, render/media keys | Encrypted `SealedSecret` resources under `overlays/production/secrets/` | Controller-generated Secrets referenced by the apps |
| Coturn configuration including shared secret | Encrypted `mahfel-coturn-config` | Secret file mounted into Coturn |
| Registry credentials and ingress TLS | Encrypted `mahfel-registry` and `mahfel-ingress-tls` | Docker config/TLS Secrets |
| Sealing private keys | Controller key Secrets in kube-system; protected off-server backup | Never committed to Git |

Previously the runbook created plaintext Kubernetes Secrets manually. Production now has a Sealed
Secrets workflow: Argo CD applies ciphertext and the controller creates the same Secret names.
Secret templates still contain placeholders only. Actual ciphertext cannot be generated until the
target cluster controller exists and real credentials are supplied; the initial secrets Kustomization
is deliberately empty. Do not sync application pods until all six encrypted resources are present.

Only Keycloak and object storage remain external. Coturn is managed in the production profile on
the VPS node; `turn.rastai.ai` must resolve to that VPS.

## Bootstrap and sealing

1. Install Argo CD privately using [its setup guide](../argocd/README.md).
2. Apply `deploy/argocd/platform` and manually sync the `sealed-secrets` Application. The chart is
   pinned to 2.20.0 (controller 0.40.0), with a dedicated platform AppProject and resource limits.
3. Install the matching reviewed `kubeseal` binary on the operator workstation.
4. Fetch the public certificate from the intended cluster and record its fingerprint:

   ```bash
   mkdir -p .local/kubernetes
   chmod 700 .local/kubernetes
   kubectl config current-context
   kubeseal --controller-name sealed-secrets --controller-namespace kube-system \
     --fetch-cert > .local/kubernetes/sealed-secrets.pem
   openssl x509 -in .local/kubernetes/sealed-secrets.pem -noout -fingerprint -sha256
   ```

5. Populate the three env template copies, `turnserver.conf`, `registry.json`, `tls.crt` and `tls.key`
   in `.local/kubernetes/`. Restrict private files to mode 0600. Use the real node IP, shared TURN
   secret, matching render keys and database/Redis/AMQP credentials. TLS must cover both web/API hosts.
6. Run the local sealing helper:

   ```bash
   ./deploy/kubernetes/seal-secrets.sh .local/kubernetes/sealed-secrets.pem
   kubectl kustomize deploy/kubernetes/overlays/production/secrets
   ```

   It uses strict name/namespace scope for `mahfel`, rejects preparation placeholders and checks
   required credentials, connection URL passwords, media HMAC, realm and shared keys. Plaintext passes through a local pipe; only encrypted output is written
   to the tracked directory. JSON output is valid YAML. It does not apply resources, commit or push.
7. Review the six encrypted files and Kustomization, commit/push them, then bootstrap their creation:

   ```bash
   kubectl -n mahfel apply -k deploy/kubernetes/overlays/production/secrets
   kubectl -n mahfel wait --for=condition=Synced sealedsecret --all --timeout=2m
   kubectl -n mahfel get secrets
   ```

   This first bootstrap allows the infrastructure/RabbitMQ preparation before full application sync.
   Argo CD subsequently owns the encrypted resources during normal full synchronization.
8. Continue the [production runbook](RUNBOOK.md), then review/manual-sync `mahfel-production`.

Argo CD's application project allows SealedSecrets, not plaintext Secret manifests. The controller
requires cluster permissions to reconcile Secrets; its separate platform project is limited to the
controller chart. Git access and the initial Argo admin credentials still require bootstrap setup
outside this application: don't create a circular dependency on the repository it cannot yet read.

## Changes and pod rollout

**Non-secret setting:** edit the env/TOML file, commit/push, review the Argo diff, then full-sync.
Kustomize changes the ConfigMap name hash and pod references, which triggers replacement of affected
pods. The shared backend ConfigMap may affect API, media and worker together; schedule accordingly.
The mounted API TURN file uses subPath, but a changed generated ConfigMap triggers a new API pod.

**Secret:** update protected local source files, re-run sealing, commit/push, then full-sync. Wait for
SealedSecret `Synced=True` before restarting the consuming pods. Secret names stay stable; `envFrom`
values are read only at process startup. Secret updates alone do not restart Deployments. Coturn also
needs a controlled restart to reread its config; avoid relayed calls during that change. Full resealing
produces fresh ciphertext for all six files even when some plaintext is unchanged; review which
underlying values changed before choosing pods to restart.

```bash
# Example: after a render-key update has successfully unsealed, restart its consumers.
kubectl -n mahfel rollout restart deployment/mahfel-web deployment/mahfel-worker
```

No automatic restart controller is installed. Database/AMQP/Redis password rotation must update the
live server identities as well as connection strings; startup env changes alone do not rotate existing
PostgreSQL/RabbitMQ users. Coordinate server changes, unsealing and consuming pod rollout.

TLS renewal updates the stable ingress Secret, which Traefik watches; ordinary certificate renewal
does not require restarting the web/API pods. Registry credentials are used for subsequent image
pulls. These differ from process environment credentials.

**Web browser API URL:** compiled into the web image using `MAHFEL_API_URL_PRODUCTION` in CI.
Changing web runtime env cannot replace it; build/publish a new image and update its deployment digest.

Never change a sealed resource's name/namespace without resealing to that identity. Do not put real
secrets in the public env ConfigMaps, examples, GitLab public build variables or commit messages.

## Backup and recovery

Encrypted Git files are insufficient to recover plaintext after losing all controller keys. Back up
**all** sealing-key Secrets after bootstrap and after key renewal, encrypt the backup and keep it
outside the VPS and outside Git:

```bash
umask 077
kubectl -n kube-system get secret -l sealedsecrets.bitnami.com/sealed-secrets-key \
  -o yaml > .local/kubernetes/sealed-secrets-keys.yaml
```

Transfer that protected backup through the existing backup mechanism; never print its contents.
On replacement, restore the controller key Secrets into kube-system before running the controller,
or restart the controller after key restoration so it reloads them. Verify it can decrypt existing
ciphertext before syncing workloads. A fresh controller without restored keys requires resealing
all resources from separately protected original credentials. Key renewal does not rotate service
passwords; these are different operations.

References: [Sealed Secrets](https://github.com/bitnami/sealed-secrets),
[Argo CD secret management](https://argo-cd.readthedocs.io/en/stable/operator-manual/secret-management/).
