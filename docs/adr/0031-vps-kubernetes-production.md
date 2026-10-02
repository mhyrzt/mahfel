# ADR 0031: Initial production on VPS Kubernetes with Argo CD

## Status

Accepted. Supersedes ADR 0013's initial hosting assumption of managed PaaS infrastructure.

## Context

For the initial production phase, the owner selected one dedicated VPS with K3s and Argo CD.
Keycloak, S3-compatible object storage and the existing Coturn VPS remain external.

## Decision

Keep all four application deployment boundaries and state ownership unchanged. Use generic
Kubernetes application resources with a `production` overlay for the single-node K3s setup and an
isolated `local` profile for development fixtures. Run PostgreSQL and RabbitMQ with infrastructure
PVCs; Redis remains replaceable. Application containers use only temporary spool/scratch storage.

GitLab CI validates and publishes images. Argo CD deploys reviewed image/configuration changes from
Git with manual sync initially. Run a controlled migration hook before application rollout; never
run development seeds in production. CI has no deployment jobs.

Media uses a single replica with direct RTC host ports and a public announced address. HTTP ingress
serves web/API; media control and operations endpoints remain private. Media rollouts require an
operational quiet period until automated drain behavior is independently qualified.

## Consequences

Node-local infrastructure volumes require off-server backups and tested restoration. One VPS is
not highly available; calls and services stop on node failure. Argo CD reconciliation is not a backup
or a zero-downtime media rollout. The initial resource budget is an estimate that requires load testing.

The reusable constraints from ADR 0013 remain: PostgreSQL/S3 own durable business state, deployment
networking stays outside domain logic, and application replicas do not rely on durable local files.
