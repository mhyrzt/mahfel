# Fresh Ubuntu 24.04 VPS bootstrap

`bootstrap.sh` prepares an **x86-64 Ubuntu 24.04 LTS VPS with systemd** for the
production Kubernetes profile. Run it from a checkout on the intended VPS. Keycloak and
object storage remain external. Mahfel workloads and Coturn are deployed later through
the [production runbook](../kubernetes/RUNBOOK.md).

## Review and apply

Choose exact compatible K3s and Argo CD releases after reviewing their release notes and
Argo CD's Kubernetes support. Versions are required inputs; the script never selects `latest`.
The VPS needs `python3` and `iproute2` for preflight checks, normally present on the server image.

```bash
# Replace every placeholder. Include your current SSH client's IP in ADMIN_CIDR.
PUBLIC_IP='<actual-public-ipv4>'
ADMIN_CIDR='<administrator-ipv4/32-or-private-vpn-cidr>'
K3S_VERSION='<exact-vX.Y.Z+k3sN>'
ARGOCD_VERSION='<exact-vX.Y.Z>'

./deploy/vps/bootstrap.sh --plan \
  --public-ip "$PUBLIC_IP" --admin-cidr "$ADMIN_CIDR" \
  --k3s-version "$K3S_VERSION" --argocd-version "$ARGOCD_VERSION"

# Review the plan and upstream installers before applying on this VPS.
sudo ./deploy/vps/bootstrap.sh --apply \
  --public-ip "$PUBLIC_IP" --admin-cidr "$ADMIN_CIDR" \
  --k3s-version "$K3S_VERSION" --argocd-version "$ARGOCD_VERSION"
```

Use `--ssh-port PORT` when SSH listens on a different port and `--node-name NAME` when
needed. If `SSH_CONNECTION` is available, its client address and server port are checked
against these inputs. Confirm them manually when sudo removes that environment variable.
Plan mode validates inputs and prints the steps without requiring root, changing files or
contacting a cluster. Apply rejects unsupported operating systems before host changes.

The six setup phases install OS prerequisites, configure kernel networking and the firewall,
install K3s, prepare ingress/storage/DNS, install Argo CD, and bootstrap Sealed Secrets.
K3s uses an explicit root-only kubeconfig, encryption at rest, the media node label, and the
selected public IPv4. The default cluster/service CIDRs are `10.42.0.0/16` and `10.43.0.0/16`;
use a separately reviewed configuration if these overlap an existing network.
Only the Sealed Secrets platform Application is created and synced. Application deployment
remains pending actual production configuration, encrypted credentials and image digests.

## Firewall

Default `--firewall ufw` allows administrator SSH/Kubernetes API access from `ADMIN_CIDR`
and public HTTP/HTTPS, RTC and TURN ports from the runbook. It also permits local CNI traffic
and forwarding for ingress/media host ports. It never resets rules. An existing active UFW
policy without the bootstrap ownership marker, firewalld, or nftables service requires manual
integration; use `--firewall external` to keep firewall configuration operator-managed.

Configure any provider firewall separately. Keep a provider console available while changing
host networking. Verify actual incoming connections, pod egress, HTTP redirect, private-port
restrictions and TURN/media routing on this VPS before proceeding. `--firewall external`
does not establish or verify those rules. Flannel 8472 and application operations/database
ports must remain inaccessible from the Internet.

## Output and recovery

Console output has six numbered headings, success/failure marks and elapsed times. Long
steps report progress every 20 seconds. Color is used only on a supporting terminal; set
`NO_COLOR=1` to disable it. Normal installer output stays in root-only files under
`/var/log/mahfel-bootstrap/`. Failures identify the phase and log path without printing
arbitrary installer output or credentials. Inspect logs privately, and redact before sharing.

After a successful bootstrap:

| File | Use |
| --- | --- |
| `/etc/rancher/k3s/k3s.yaml` | Root-only administrator kubeconfig |
| `/var/lib/mahfel/bootstrap/sealed-secrets.pem` | Public sealing certificate for the operator's sealing workflow |
| `/var/lib/mahfel/bootstrap/sealed-secrets-keys.yaml` | Private controller recovery keys; encrypt and back up off-server |

The output prints a localhost-bound Argo CD port-forward command. Access it through an SSH
tunnel. Retrieve administrator credentials privately using the
[Argo CD instructions](../argocd/README.md); the bootstrap never prints them.

Rerun using the same inputs after a transient failure. Existing K3s version/configuration and
Argo CD server version must match; upgrades and cluster adoption need a separate operation.
Reruns preserve controller keys and cluster data. Recovery files are replaced only after
successful retrieval and parsing. Completed phases are retained after a failure; setup is
not a transaction with automatic rollback. Keep the chosen versions and CLI inputs in the
deployment record. Periodically refresh and back up controller keys after key rotation.

Continue with [production configuration](../kubernetes/CONFIGURATION.md) and the
[runbook](../kubernetes/RUNBOOK.md), including DNS, external dependencies, secret sealing,
infrastructure preparation, application sync and live acceptance checks.

## Local checks

```bash
bash -n deploy/vps/bootstrap.sh
shellcheck deploy/vps/bootstrap.sh
python3 deploy/vps/tests/test_bootstrap.py
```

These checks exercise CLI validation and isolated functions with command stubs. They do not
install packages, alter the host firewall or establish real cluster/RTC connectivity.
