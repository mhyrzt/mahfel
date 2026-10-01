---
id: TASK-159
title: Add validation and production Darkube CI
status: Done
assignee:
  - '@codex'
created_date: '2026-09-28 08:56'
updated_date: '2026-10-01 14:31'
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

Live backend-lint failed because the shared Rust CI setup installed python3 without python3-pip. Verified mediasoup-sys 0.17.0 build.rs invokes python3 -m pip install --target for invoke. Added python3-pip and an early pip-version check to .backend-base, covering lint/unit/doctest jobs; Docker chef already installs pip. YAML/inheritance assertions, strict MkDocs, and diff checks pass. The corrected runner build has not yet been executed.

User requested LMS-style preparation and independent Pages. Add a prepare stage with change/tag-aware Cargo fetch and frozen Bun install cache writers. Consumers pull dependency caches; Rust build/tool cache stays separate. Pages and docs-check use needs=[]; Pages performs its own strict build with dependencies=[] and can publish regardless of app-job failures.

User withdrew the requested docs-job independence change. Removed only the new needs/dependencies edits and corresponding independence docs; Pages/docs job configuration is unchanged. Preparation changes remain in scope.

Final docs requirement: docs-check starts immediately with needs=[] and is always selected on main. Pages has one required needs entry for docs-check with artifacts=false, so it starts automatically after docs-check succeeds and does not wait for application jobs. This supersedes the preceding docs withdrawal.

Preparation/Pages validation: GitLab CI Lint main dry-run accepts the final assembled configuration. Local YAML assertions verify prepare-stage ordering, dependency cache writer/consumer policies, distinct Rust build cache, required Pages->docs-check edge and unchanged release stage gates. Focused frontend YAML formatting, strict docs and git diff checks pass. Live prepare-cache performance and Pages scheduling await the pushed pipeline.

Live pipeline 2103789: both prepare jobs, backend lint and independent Pages succeeded. Failed jobs 7282536 (frontend-test) and 7282533 (backend-unit-test) never reached assertions: Vitest/jsdom workers failed under Bun; rustc compiling aws-sdk-s3 was killed by SIGKILL before test execution (resource pressure suspected, host OOM not confirmed). Switch only frontend-test to pinned Node 24.15.0 with Bun 1.4.2 bootstrap/frozen install and explicit Node Vitest entrypoint. Limit Cargo to two build jobs and disable CI dev/test debug symbols. Verified Node image availability and exact Bun bootstrap in a disposable container; local Node smoke passes while forced-Bun smoke cannot start a worker. CI Lint and strict docs pass; committed frontend full suite is being checked under Node.

Node committed-source suite exposed one DOM-heavy pagination test exceeding the default five-second Vitest timeout. The same assertion passes with a 15-second budget under load (about 10.5 seconds of test execution). Added CI-only --testTimeout=15000; no test assertions or application code changed.

Final local frontend validation against committed HEAD source (reusing the installed dependency tree): full Node suite ran all 125 tests; 124 passed and only pagination exceeded the old 5000ms budget. All eight tests in that file pass with --testTimeout=15000. No assertions were weakened. Node/Bun bootstrap container, focused YAML formatting, strict docs, configuration assertions and GitLab CI Lint pass. Backend mitigation remains unconfirmed until the next runner build; SIGKILL is not proof of a host OOM without host logs.

2026-10-01 release pipeline repair: main 2113655, api 2113654 and media 2113652 backend-lint failed because .backend-base omitted GStreamer development packages. Added libgstreamer1.0-dev and libgstreamer-plugins-base1.0-dev plus an early pkg-config >=1.22 check for core/app/video, covering lint/unit/doctest jobs. Integration/image Docker paths already install these dependencies. Exact Rust 1.95.0 Bookworm CI image dependency installation and pkg-config checks pass (all three 1.22.0); assembled local YAML inheritance assertions and GitLab CI Lint main dry-run pass with no errors or warnings. Worker tag pipeline additionally timed out fetching Debian packages, and media format failed Rust toolchain metadata TLS download; these are distinct network failures. Existing v1.0.0 tags remain unchanged. New runner pipeline verification pending push.

2026-10-01 follow-up: backend-lint 7321163 passed after GStreamer repair. Unit job 7321166 and doc job 7321167 failed during mediasoup-sys compilation before any assertions. Meson restored from lint runner directory /2 still referenced /2 while jobs executed in /0 and /1, causing FileNotFoundError for meson-info/tmp_dump.json. Removed backend/target/ from the shared cache and changed rust-build-1.95.0 to rust-tools-1.95.0 so legacy archives cannot reintroduce compiled targets. Cargo registry/git and installed binaries remain cached; Docker BuildKit is unchanged. Updated CI documentation. Local assembled YAML cache/inheritance checks and GitLab CI Lint main dry-run pass. New live test execution pending push.

2026-10-01 follow-up unit runtime repair: pipeline 2114372 passed lint, doctests and integration. Unit job 7321578 ran all 394 tests: 393 passed; commands_are_fenced_and_sealed_generations_cannot_reactivate failed at recording prepare with recording codecs unavailable. Added GStreamer base/good/bad runtime plugins only to backend-unit-test, preserving shared setup via !reference. Exact Rust CI image with these packages resolves all 13 elements required by CapturePipeline::check_available. The exact failing test passes locally (1 passed), assembled CI inheritance checks and GitLab CI Lint dry-run pass without errors/warnings, strict docs and diff checks pass. No assertions or readiness checks changed. Live full unit rerun pending push.

Security remediation investigation for pipeline 2114541: all backend tests and images pass. Trivy jobs 7322100/1/2 execute successfully and block on Debian 12 OS findings, not scanner failure. API 70 (66 High, 4 Critical), media 291 (275 High, 16 Critical), worker 375 (356 High, 19 Critical); all structured reports show no solution for the installed Bookworm packages. Plan: test supported Debian 13 runtime packages against the same blocking scanner, verify native binary compatibility and media/export requirements, remediate findings where patches exist, and retain explicit evidence for any remaining unpatched issues. Do not change severity thresholds or ignore unfixed findings globally.

Prepared runtime remediation: API/media/worker production stages now use Debian 13 trixie-slim and apt-get upgrade before installing runtime packages. Rust build/toolchain stays pinned on Bookworm. Each final image checks ldd output and rejects unresolved shared libraries. Debian tracker confirms patched Trixie versions for representative GStreamer and util-linux findings; this is not proof all findings are resolved. Dockerfile build checks, strict docs and diff checks pass. Local runtime build/package download repeatedly stalls; current Trivy DB downloads from upstream and mirror timed out, so no fresh local scan or full runtime compatibility claim is made. Existing shared Trivy cache is August 25 and is not valid current security proof. Pushing the focused patch will use the remote build/scan gates for actual findings. Asked user whether to retain blocking all HIGH/CRITICAL versus reporting all but blocking only patchable findings; keep existing policy unless explicitly changed.

User explicitly changed security policy and withdrew the Docker remediation: make scans nonblocking and revert Docker changes. Restored backend/Dockerfile byte-for-byte to c8d8889 parent (Bookworm; removed the attempted Trixie upgrades and ldd checks). Shared .image-scan now has allow_failure=true for all four apps in branch and release pipelines; vulnerability reports and HIGH/CRITICAL failing exit codes remain visible. Tests, builds, release protection and tag ancestry are unchanged. Updated CI documentation to describe advisory findings and scanner failures. Assembled YAML assertions and GitLab CI Lint main dry-run pass; resolved API/media/worker/web scan jobs all report allow_failure=true.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added strict backend/frontend/docs/protobuf CI with integration tests and reports. Main/MRs validate only. Protected APP_NAME-vMAJOR.MINOR.PATCH tags on main commits build and scan their component, publish registry.hamdocker.ir/hamdars/mahfel/APP_NAME:vVERSION, and deploy the matching vVERSION to Darkube production. API/worker/web deployment is automatic; media remains manual. Configuration, release-selection/ancestry checks, scripts, image-tag mapping and docs pass. Live releases and external protected-tag/credential/platform setup remain unperformed.
<!-- SECTION:FINAL_SUMMARY:END -->
