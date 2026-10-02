---
id: TASK-171
title: Prepare in-cluster Coturn and document production GitOps configuration
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 13:04'
updated_date: '2026-10-02 13:18'
labels: []
dependencies: []
ordinal: 179000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prepare the production Kubernetes/Argo CD profile after removal of the old Coturn VPS. Run Coturn in-cluster; keep only Keycloak and object storage external. The owner chose Sealed Secrets for encrypted GitOps credentials. Provide bootstrap, sealing, rollout and key-recovery preparation without deploying or generating real production secrets.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Production profile includes one Coturn replica on the selected node with direct TURN and bounded relay ports, secret-backed configuration and resource limits
- [x] #2 Runbook describes new TURN DNS/firewall/configuration and no longer assumes an existing Coturn VPS
- [x] #3 Production manifests and documentation validate; live TURN connectivity remains an explicit deployment gate
- [x] #4 Sealed Secrets controller GitOps bootstrap and strict-scope sealing workflow replace routine manual secret provisioning; controller-key recovery is documented
- [x] #5 GitOps documentation explains public configuration, Sealed Secrets provisioning, rollover behavior and bootstrap/restore requirements
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add production Coturn with mounted secret config and direct node networking. 2. Add Sealed Secrets controller Argo bootstrap and strict-scope encryption helper. 3. Wire encrypted resources and document configuration rollouts/key backup. 4. Update topology decision and validate manifests/scripts/docs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared Coturn host-network deployment with mounted configuration Secret, auth-only allocation, media peer allowlist and bounded 49160-49359 relay ports. User selected Sealed Secrets: official chart 2.20.0/controller 0.40.0, separate Argo platform project, strict-scope sealing helper for six credentials/TLS/config resources, required-key/shared-password/HMAC checks and offline SealedSecret schema. Production schema lint passed for 26 resources, Argo schemas for four objects, controller Helm/RBAC render and ten built-in resources validated (CRD definition excluded from generic lint; extracted custom schema validated). Temporary-key tests passed encryption/recovery for six Secret types, namespace binding and rejection preserving output. Coturn config starts in isolated cached-image container without networking/public ports. ShellCheck, docs, local links and diff checks passed. No real cluster deployment, real credentials or ciphertext generation occurred; controller-key backup, DNS/firewall and live TURN acceptance remain rollout requirements.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Moved production Coturn preparation into Kubernetes and adopted the approved Sealed Secrets GitOps workflow. Updated topology ADR, production runbook and configuration guide, including encrypted secret generation, stable Secret references, rollout behavior and key recovery. Relevant offline/schema, sealing/recovery and isolated Coturn startup checks passed. Target-cluster bootstrap and live connectivity remain deployment gates.
<!-- SECTION:FINAL_SUMMARY:END -->
