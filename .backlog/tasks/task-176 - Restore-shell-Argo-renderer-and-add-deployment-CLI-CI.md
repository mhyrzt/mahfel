---
id: TASK-176
title: Restore shell Argo renderer and add deployment CLI CI
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 19:36'
updated_date: '2026-10-02 19:44'
labels: []
dependencies: []
ordinal: 184000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Restore a separate render.sh for Argo CMP using the existing repo-server image, without Python dependencies or a custom image in the cluster. Keep the operator CLI Python renderer and TOML settings. Add blocking CLI lint, typing, crypto/bootstrap tests and package build jobs to GitLab CI; update docs/config/tests and leave changes uncommitted.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Argo uses a mounted shell renderer and standard repo-server image with protected SOPS key and isolated staging
- [x] #2 Custom CMP Dockerfile, image settings and obsolete executable are removed; operator CLI retains Python rendering and TOML config
- [x] #3 GitLab CI runs locked CLI quality, real crypto/bootstrap tests and package builds for relevant changes without production configs or deployment credentials
- [x] #4 Both renderers have focused parity/failure checks; package, shell, CI YAML, typing and docs validation pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Restore shell CMP rendering and native Python setup of its ConfigMap/standard-image sidecar. 2. Remove custom-image settings and update focused tests/docs. 3. Add GitLab CLI validation and package jobs with pinned test tools. 4. Validate render parity, crypto/failure cleanup, CI config, package and docs; record live CI boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Restored deploy/argocd/render.sh as the standalone shell CMP renderer; native configure-sops mounts it with plugin.yaml and discovers the existing repo-server image. Removed the custom CMP Dockerfile, image/pull-secret TOML fields and mahfel-render executable. Operator rendering remains Python. Both renderers now have identical-output, missing/wrong/tampered-secret, partial-output and SIGTERM-cleanup coverage. Added deploy/cli/.gitlab-ci.yml with root include, locked uv prepare/cache, blocking Ruff/mypy/shell checks, checksum-verified SOPS/kubectl crypto/bootstrap test prerequisites, JUnit reports and wheel/source artifacts. No image release job or new tag pattern is needed with the restored standard-image shell CMP. Existing application publishing/scan policy is unchanged. Validation: 32 tests passed; Ruff/format, strict mypy (10 source files), shell syntax/ShellCheck, wheel/source build and entrypoints/helper contents, strict docs, Compose config and git diff --check passed. Local assembly verified every CI include, extends/reference, stage and needs plus blocking CLI rules. GitLab server lint/live pipeline were not run because glab has no token in this environment. No cluster changes, publication, commit or push.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Restored separate Argo shell rendering using the existing Argo image and retained TOML-backed Python operator commands. Added blocking deployment CLI validation/package CI with disposable configs and keys. All focused local checks passed; remote GitLab pipeline validation remains unverified. Changes remain uncommitted.
<!-- SECTION:FINAL_SUMMARY:END -->
