# Hamravesh / Darkube deployment

Mahfel targets stateless PaaS-friendly containers. `mahfel-web`, `mahfel-api`, and `mahfel-worker`
may scale conventionally. Media nodes and coturn require explicit public network design because
WebRTC depends on address/port reachability rather than HTTP routing alone.

## Required topology

- Public HTTPS ingress routes browser traffic separately to web and API.
- gRPC 50051 and operations 9101–9103 remain private and policy-restricted.
- PostgreSQL, Redis, RabbitMQ, S3, Prometheus, Grafana, and management ports remain private.
- Each media replica advertises an address reachable by browsers and exposes its configured RTC
  UDP ports without remapping. TCP on those ports is a useful fallback, not the preferred path.
- coturn owns a stable public address, 3478 UDP/TCP, a production-sized UDP relay range, and valid
  TLS configuration for 5349 and optionally 443.
- No application container depends on a persistent filesystem volume.
- The image includes non-secret `default.toml` and `docker.toml` baselines. Darkube deployment
  configuration must override credentials, issuer URLs, announced addresses, and other environment
  specific values through environment variables/secrets; do not depend on an arbitrary file mount
  for a Docker-image app. Confirm the linked Darkube custom-config behavior with platform support
  before introducing mounted configuration files.

## Platform validation gates

The following are unresolved until tested with Hamravesh/Darkube engineering and a production-like
environment:

1. Does an external UDP port preserve the same container port exactly?
2. What sustained bandwidth and packets-per-second limits apply to UDP load balancers?
3. Is client source IP preserved, and through which network layers?
4. How does generic UDP load balancing maintain flow affinity?
5. Can or should every media replica receive an individual stable public IP?
6. How are multiple RTC ports declared, health-checked, and drained during rollout?

Do not launch production RTC on assumptions here. Validate direct UDP with at least two media nodes,
then deliberately block direct connectivity and validate TURN fallback. Capture packet loss, RTT,
retransmissions, ingress/egress, PPS, and load-balancer behavior.

## Rollout behavior

Protected component version tags on main commits publish validated images and update the matching
existing API, worker, or web Darkube app. Media tags publish its image and offer a manual rollout job
until its networking and drain requirements below are qualified.
App IDs, protected deploy tokens, registry access, and migration prerequisites are documented in
[continuous integration](../development/ci.md#darkube-deployment). Production app provisioning and
platform qualification remain M16 work; the CI jobs do not supply those prerequisites.

Readiness removes API/worker traffic or work without triggering liveness restarts. A media rollout
first marks a node draining in Redis, stops new room placement, waits for rooms to end (or reaches a
documented deadline), and then terminates gracefully. `announced_address` is deployment
configuration, never domain logic. Secrets belong in the platform secret store and must not be
baked into images.
