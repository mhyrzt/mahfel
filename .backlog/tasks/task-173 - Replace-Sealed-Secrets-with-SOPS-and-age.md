---
id: TASK-173
title: Replace Sealed Secrets with SOPS and age
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 15:01'
updated_date: '2026-10-02 15:30'
labels: []
dependencies: []
ordinal: 181000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use SOPS age encryption for production secrets with a restricted Argo CD CMP sidecar, Ubuntu VPS bootstrap, local encryption/render helpers and recovery documentation. Remove obsolete controller assets; preserve four application boundaries and manual GitOps sync.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Production credentials encrypt with age-backed SOPS and plaintext is not stored in Git
- [x] #2 Argo CD decrypts only through the configured CMP sidecar with a separately bootstrapped private key
- [x] #3 Ubuntu VPS bootstrap preserves age identity on rerun and installs the SOPS integration without deploying unconfigured apps
- [x] #4 Controller-specific assets and active guidance are replaced and a superseding ADR records the decision
- [x] #5 Focused crypto/render/failure tests, shell and deployment/documentation checks pass; real cluster proof remains explicit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace controller assets with SOPS age key bootstrap and a restricted Argo CMP sidecar. 2. Reuse credential validation while encrypting one six-Secret bundle; decrypt into private temporary files before Kustomize. 3. Update VPS plan/apply and local render/lint commands. 4. Add superseding ADR and update active docs. 5. Verify real age crypto, wrong-key/tamper/error paths, render/schema, shell and docs without applying to a cluster.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced controller assets/custom schema/sealing helper with SOPS v3.13.3 and age. Encryption helper retains existing credential/HMAC/realm checks and publishes one encrypted six-Secret List after successful validation/encryption. Local/CMP renderer decrypts to private scratch before Kustomize and buffers output to avoid partial Secret emission. Argo App explicitly selects mahfel-sops; namespace-scoped Project allows ordinary Secrets. configure-sops.sh bootstraps a key Secret plus CMP ConfigMap and strategic repo-server patch: nonroot sidecar, read-only age mount only there, isolated memory-backed /tmp, no Git credentials, SOPS init image, same Argo image as repo-server. Key mismatches are rejected. Ubuntu bootstrap installs age and checksum-pinned host SOPS, preserves root-only age identity and supports recovery. ADR 0033 supersedes only ADR 0032 secret management; active docs/runbooks and render/lint commands updated. Validation: five real crypto/render/CMP tests plus eight VPS tests passed, 32 production fixture resources and two Argo resources passed strict schemas, local profile rendered, deployment YAML/links, ShellCheck/syntax, strict docs and whitespace checks passed. No real production credentials, cluster apply, image pull/CMP runtime or VPS firewall/network proof was performed. Implementation and updated docs remain uncommitted for user review.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Prepared SOPS/age production GitOps replacement with guarded identity bootstrap, restricted CMP decryption, offline encryption/render helpers and recovery guidance. Local crypto, failure-path, schema and docs validation passed; actual cluster/VPS runtime remains deployment verification.
<!-- SECTION:FINAL_SUMMARY:END -->
