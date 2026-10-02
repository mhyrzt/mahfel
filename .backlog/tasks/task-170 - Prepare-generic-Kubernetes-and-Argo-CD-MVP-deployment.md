---
id: TASK-170
title: Prepare Kubernetes and Argo CD production deployment
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 12:37'
updated_date: '2026-10-02 14:53'
labels: []
dependencies: []
ordinal: 178000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace Minikube-only layout with generic Kubernetes deployment preparation. MVP runs on one K3s VPS with external Keycloak, S3-compatible storage and Coturn, without the former deployment platform deployment. Preserve local development fixtures separately.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Local Kubernetes workflow relocates without broken paths
- [x] #2 MVP manifests render and validate with private infrastructure, external dependency configuration, resources and controlled migrations
- [x] #3 Argo CD bootstrap and manual GitOps rollout are documented and prepared without plaintext production secrets
- [x] #4 VPS MVP runbook covers provisioning, ingress TLS, media networking, backups, rollout and acceptance; the former deployment platform deployment is disabled by default
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Move development fixtures into deploy/kubernetes/overlays/local. 2. Prepare base applications and production overlay for external Keycloak/storage/Coturn. 3. Add K3s configuration and Argo CD bootstrap/manual sync. 4. Supersede hosting ADR, remove the former deployment platform CI deployment jobs, and document VPS production operations. 5. Validate manifests, shell scripts, CI references and documentation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
User requested production naming: profile overlays/production and Argo Application mahfel-production. Strict schema checks passed for 25 production and 59 local resources; 2 Argo resources validated against official CRD schemas. ShellCheck, Kustomize reference/invariant checks, relocated fixture preservation, local CI includes/extends/dependencies and strict MkDocs build passed. No live cluster operations or deployment were performed. Endpoints, TLS/secrets, supported bootstrap versions, registry image availability/digests and live acceptance remain operator prerequisites. Existing TASK-169 changes and user notes preserved.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Prepared generic Kubernetes production/local profiles, single-VPS K3s configuration, Argo CD manual GitOps deployment with migration ordering and PVC protections, external-service secret/configuration templates, and the production RUNBOOK. Removed active the former deployment platform deployment jobs while preserving build/publish. Offline manifests and Argo schemas, shell scripts, CI references and documentation validated; live rollout is explicitly outside preparation.
<!-- SECTION:FINAL_SUMMARY:END -->
