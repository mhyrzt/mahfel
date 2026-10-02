# Architecture decision records

ADRs are immutable decision snapshots. Accepted records are superseded by a new ADR rather than
silently rewritten. Minor clarity and link corrections are allowed without changing the decision.

Records 0001–0027 establish Mahfel's bootstrap boundaries. Status is **Accepted** unless stated
otherwise.

- [ADR 0028: Recording MP4 exports](0028-recording-mp4-export.md)
- [ADR 0029: Mandatory recording start](0029-mandatory-recording-start.md)
- [ADR 0030: Native recording image rendering](0030-native-recording-image-rendering.md)

- [ADR 0031: VPS Kubernetes production](0031-vps-kubernetes-production.md) supersedes the initial hosting assumption in ADR 0013.

- [ADR 0032: In-cluster Coturn and Sealed Secrets](0032-cluster-turn-and-sealed-secrets.md) updates ADR 0031 dependency placement and secret management.
