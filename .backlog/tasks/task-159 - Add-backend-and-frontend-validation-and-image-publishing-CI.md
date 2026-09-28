---
id: TASK-159
title: Add validation and production Darkube CI
status: Done
assignee:
  - '@codex'
created_date: '2026-09-28 08:56'
updated_date: '2026-09-28 09:53'
labels:
  - ci
  - backend
  - frontend
dependencies: []
ordinal: 167000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add strict Rust/Bun/docs/protobuf CI and disposable integration tests. Protected APP_NAME-vMAJOR.MINOR.PATCH tags on main commits select per-component registry.hamdocker.ir publishing and Darkube production rollout. Main/MRs validate only. API/worker/web tagged releases deploy automatically; media tagged releases expose an optional manual deploy pending platform qualification. Preserve docs Pages and document runner, tag protection, registry, credentials, versioning, and migration prerequisites.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Change-aware MR and branch pipelines run strict backend/frontend checks with dependency caches and test reports
- [x] #2 CI configuration validates and runner, registry, and local verification requirements are documented
- [x] #3 Protected component version tags select only their own production image and Darkube release; main pushes never publish or deploy
- [x] #4 The selected component image has blocking build and scan jobs before publishing; Docker uses APP_NAME:vVERSION and Darkube receives the same vVERSION image tag
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Protected APP_NAME-vMAJOR.MINOR.PATCH tags on commits reachable from main select per-component Darkube releases. Main/MRs validate only. Strip APP_NAME- for Docker APP_NAME:vVERSION publishing, cleanup, and Darkube image-tag. Preserve blocking validation/scans and require the production API URL for web releases.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented component CI includes, shared image archive/build/scan/publish jobs, change-aware workflow, strict docs/proto validation, JUnit reports, pinned toolchains, caches, dedicated host-network Docker integration target, bounded Vitest workers, release API URL selection, and cleanup. Publishing targets registry.hamdocker.ir with branch-slug/full-SHA tags and stage gates; no deploy or promotion bypass.

Validation: GitLab CI Lint dry-run with main context accepted the assembled config. GitLab API validation also returned valid=true with no errors/warnings and resolved build/scan/publish jobs before final cleanup/tag/docs refinements; final CLI dry-run accepted those refinements. ShellCheck passes both scripts; six public-URL selection scenarios and Docker-stub pass/fail/JUnit/cleanup scenarios pass. Docker build --check passes backend ci-integration and frontend runtime targets; pinned Trivy mirror manifest exists. Buf format/lint and strict MkDocs pass. nextest CI profile passes 88 domain tests and writes valid JUnit. Frontend lint/typecheck passed at the earlier working-tree snapshot; Vitest with two workers writes JUnit with 115/117 passing and two concurrent chat-test failures. Later full formatting checks find concurrent edits outside this CI patch in backend Postgres and five frontend files. Those changes were preserved. Actual image compilation, Docker-backed integration execution, vulnerability scan results, registry push, and a committed GitLab pipeline run were not performed. Publishing requires runner socket access and documented registry/API URL variables.

2026-09-28 scope refinement: Mahfel has only a production release environment. Restricted all four image publishers to production branch push pipelines. Removed default/staging release URL variables and the staging workflow exception. Production branch web builds require MAHFEL_API_URL_PRODUCTION; other refs retain validation-only builds. Updated CI documentation. YAML/rule assertions, five URL-selection scenarios, ShellCheck, and changed-file formatting checks pass.

User clarified the only release environment is Darkube production. Extending the CI setup with Darkube rollout jobs; release branch selection requested. Production topology and media rollout limitations remain documented and are not live-qualified by adding CI jobs.

User explicitly selected the default branch (main) for Darkube production. This supersedes the earlier separate production-branch assumptions. Publish and deploy rules share default-branch push conditions; main web builds require MAHFEL_API_URL_PRODUCTION.

Darkube completion: verified actual v1.1 CLI deploy help supports token, app-id, image-tag, and job-id. Reset the image entrypoint for GitLab scripts. Final main-context GitLab CI Lint dry-run passes; resolved deploy inheritance selects Darkube commands, production environment/resource groups, default-branch push rules, dependencies=[], and no needs that bypass stage gates. Main/other-branch/missing-URL scenarios, ShellCheck, focused frontend YAML formatting, strict MkDocs, and changed-file diff checks pass. Main deploys API/worker/web after all scans and publishes; media is optional manual per current deployment guide. No live deploy, app creation, registry push, migration, or RTC qualification was attempted. App IDs/tokens and production runtime/migration setup must be configured on GitLab/Darkube.

Release-tag refinement supersedes automatic main publishing/deployment. Stable protected api-vX.Y.Z/media-vX.Y.Z/worker-vX.Y.Z/web-vX.Y.Z push tags select only their runtime image and release jobs. Backend tags retain shared workspace checks; every release checks main ancestry. Exact CI_COMMIT_TAG is used for Docker and Darkube. Main builds use validation fixture URL; web release builds require MAHFEL_API_URL_PRODUCTION. Validation: 18 local pipeline-selection contexts (protected/unprotected components, invalid/leading-zero/prerelease tags, main/MR/feature/schedule/UI contexts), dependency/inheritance assertions, three build URL scenarios, disposable Git ancestry acceptance/rejection, ShellCheck, focused YAML formatting, strict docs, and diff checks pass. GitLab CI Lint accepted main and four synthetic protected-component tag contexts. No actual Git tag, registry publication, deployment, or protected-tag setting was created.

Image-tag refinement: kept Git APP_NAME-vVERSION release triggers, stripped the component prefix for Docker APP_NAME:vVERSION and Darkube vVERSION, including publisher cleanup. Eight local component/version scenarios using stubbed Docker/Darkube CLIs passed publishing, cleanup, and deployment argument assertions. YAML parsing, strict MkDocs, and git diff --check passed. No images were published or deployed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added strict backend/frontend/docs/protobuf CI with integration tests and reports. Main/MRs validate only. Protected APP_NAME-vMAJOR.MINOR.PATCH tags on main commits build and scan their component, publish registry.hamdocker.ir/hamdars/mahfel/APP_NAME:vVERSION, and deploy the matching vVERSION to Darkube production. API/worker/web deployment is automatic; media remains manual. Configuration, release-selection/ancestry checks, scripts, image-tag mapping and docs pass. Live releases and external protected-tag/credential/platform setup remain unperformed.
<!-- SECTION:FINAL_SUMMARY:END -->
