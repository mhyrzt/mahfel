# Argo CD for VPS production

CI tests and publishes images; Argo CD deploys the reviewed Git configuration. CI has no deployment jobs in this setup.

1. Prepare K3s and the namespace using the [runbook](../kubernetes/RUNBOOK.md).
   Application secrets/TLS will be sealed after controller bootstrap.
2. Select an exact supported Argo CD release compatible with the installed Kubernetes version.
3. Inspect the target context, then run:

   ```bash
   ARGOCD_VERSION=<exact-release> ./deploy/argocd/bootstrap.sh --apply
   kubectl -n argocd port-forward service/argocd-server 8443:443
   ```

4. Access the UI through that tunnel; obtain the initial admin password privately from
   `argocd-initial-admin-secret`, change it, then remove that bootstrap secret. Do not expose Argo CD publicly.
5. Register `https://hamgit.ir/hamdars/mahfel.git` with read-only repository credentials through the UI/CLI.
   These credentials are separate from image-pull credentials. Never commit either credential.
6. Apply the platform project/controller Application first and manually sync `sealed-secrets`:

   ```bash
   kubectl apply -k deploy/argocd/platform
   ```

   Wait for `deployment/sealed-secrets` in kube-system to be ready. Follow the
   [sealing workflow](../kubernetes/CONFIGURATION.md), back up controller keys, and prepare infrastructure.
   Push the reviewed configuration and encrypted resources so Argo CD can actually fetch them, then apply:

   ```bash
   kubectl apply -k deploy/argocd
   ```

7. Review the diff and manually sync `mahfel-production`, including hooks. Automated sync/prune is absent.
   Selective sync skips hooks, so do not use it for an application release requiring migrations.

`application.yaml` tracks `main` but does not deploy new commits automatically. Pin `targetRevision`
to a reviewed commit for stricter approval. Update component image digests in the initial production Kustomization
as a separate reviewed deployment change; publishing a release alone does not change the VPS.

Infrastructure uses wave -2; the migration Sync hook runs at wave 0; applications and ingress follow.
PVCs and namespace have `Prune=false,Delete=false`, and the Application has no destructive deletion
finalizer. These controls are not backups. Manual kubectl deletion or disk failure can still lose data.
A failed migration stops normal full sync; inspect its logs and database state before retrying.

This uses the upstream single-instance installation: it accepts controller downtime and is not HA.
Official documentation recommends HA for production; one VPS cannot provide node failure resilience.

References: [installation](https://argo-cd.readthedocs.io/en/stable/operator-manual/installation/),
[sync hooks and waves](https://argo-cd.readthedocs.io/en/stable/user-guide/sync-waves/),
[secret management](https://argo-cd.readthedocs.io/en/stable/operator-manual/secret-management/).
