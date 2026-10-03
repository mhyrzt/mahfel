---
id: TASK-175
title: Consolidate deployment helpers in the Python CLI
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 18:44'
updated_date: '2026-10-02 18:59'
labels: []
dependencies: []
ordinal: 183000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Move manifest rendering and Argo bootstrap/SOPS setup into the uv CLI; package VPS and local deployment helpers behind TOML-configured CLI commands and remove scattered operator shell entry points. Use the same Python renderer in the CMP sidecar, preserve secret cleanup, migration hooks and bootstrap safeguards. Keep runtime container entrypoints separate and changes uncommitted.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Local CLI and Argo CMP share Python rendering with no shell renderer and no partial output on failure
- [x] #2 Argo bootstrap and SOPS configuration are CLI operations with explicit contexts and protected key handling
- [x] #3 VPS and local deployment helpers live inside the package and accept configuration through TOML-backed commands
- [x] #4 CMP image build, all callers, runbooks and focused tests are updated and obsolete entry points removed
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Implement a shared Python renderer and CMP image. 2. Add typed bootstrap/local TOML settings and Argo operations. 3. Package preserved host/local shell logic behind CLI commands. 4. Update callers/docs and run focused tests, typing, lint and image checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Shared renderer.py replaces render.sh in operator CLI and mahfel-render CMP entry point. Native Python Argo bootstrap/configure-sops use explicit kube_context and preserve cluster key mismatch rejection. VPS and Minikube Bash procedures are packaged in mahfel_deploy/scripts and called via typed TOML-backed commands, retaining existing host/firewall/context/CA safeguards. Removed scattered operator scripts and updated Just aliases, runbooks and private-context Docker ignores. Added a digest-pinned CMP Dockerfile with the Python package and Kustomize, explicit image configuration and optional bootstrap pull Secret. Validation: 31 tests passed, including real disposable SOPS crypto, bad key/tampering, render failure and SIGTERM cleanup; Ruff check/format, strict mypy (10 source files), uv wheel/sdist build and packaged-helper contents, strict docs, Compose config, TOML examples, CLI help, Just dry runs and git diff --check passed. Docker static build checks passed with no warnings. Full image build was stopped after base layers barely progressed for more than six minutes; no container runtime proof, push, host setup or live cluster changes. Implementation remains uncommitted.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Consolidated deployment operations under the uv CLI with TOML settings. Local and Argo CMP share native Python rendering; Argo bootstrap/SOPS setup are native Python, while tested host/local Bash logic is packaged behind CLI commands. Updated image build and runbooks. All focused local/static checks pass; full sidecar image/runtime verification remains pending because registry downloads were too slow.
<!-- SECTION:FINAL_SUMMARY:END -->
