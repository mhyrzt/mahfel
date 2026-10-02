# ADR 0032: In-cluster Coturn and Sealed Secrets

## Status

Accepted. Supersedes ADR 0031's external Coturn placement and manually provisioned application secrets.

## Context

The owner removed the former Coturn VPS and selected Kubernetes/Argo CD for infrastructure other
than existing external Keycloak and object storage. Sealed Secrets was selected for production credentials.

## Decision

Run one Coturn replica in the production profile on the media node with host networking, direct
TURN/relay ports, secret-backed configuration, bounded resources and controlled replacement.
The initial IPv4 setup permits relaying only to the configured media announced address. Host/provider
firewalls govern host-network ports. TLS listeners require separate certificate configuration and validation.

Bootstrap the Sealed Secrets controller with a separate restricted Argo CD platform project and a
pinned Helm chart. Encrypt production credentials, Coturn config, registry auth and ingress TLS into
strict name/namespace-scoped SealedSecrets in Git. The controller creates the stable Secret names
already referenced by workloads. Keep public configuration in generated ConfigMaps.

Argo CD continues manual synchronization initially. ConfigMap hashes trigger configuration rollout;
stable Secret updates require explicit coordinated pod replacement after unsealing. Bootstrap Git/admin
credentials and controller private keys remain outside application Git manifests.

## Consequences

A single node remains a failure boundary for media and TURN. Coturn replacement interrupts relayed
calls; no durable allocation volume is needed. A bounded relay range needs capacity and connectivity
qualification. Only Keycloak and object storage remain external dependencies.

Encrypted Git resources require controller private-key backup for recovery. Production ciphertext
can be created only after target-cluster bootstrap; repository placeholders are not deployment-ready.
An encrypted resource cannot change name/namespace without resealing. GitOps does not automate
server-side database password changes or remove the need for key backup and live acceptance.
