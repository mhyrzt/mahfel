---
id: TASK-169
title: Publish corrected release images for all apps
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-02 09:21'
updated_date: '2026-10-02 10:20'
labels:
  - release
  - ci
dependencies: []
ordinal: 177000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish api, media, worker and web v1.0.1 images from the corrected source. Preserve v1.0.0 tags, existing advisory scan policy and blocking validation/build gates. This release publishes images and GitLab release entries only; do not execute Darkube deployments. Web requires the user-provided production API URL.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Protected component release pipelines publish all four registry images with recorded digests
- [x] #2 Release pipelines do not create deployment jobs in publish-only mode
- [ ] #3 Web image uses the approved production API URL and all relevant validation gates pass
- [ ] #4 API, media, and worker versions and tags are v1.0.1; corrected web version and tag are v1.0.2, including all CI repairs.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add an explicit publish-only pipeline switch and validate release rules. 2. Set versions to 1.0.1 and regenerate API contracts. 3. Configure tag protection and required build variables. 4. Push component tags with deployment disabled and track builds/publishing to completion. 5. Verify registry digests and finalize release notes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Web v1.0.1 production and Docker builds passed, but Storybook failed because staticDirs still referenced removed recording-renderer-dist. Removed the obsolete Storybook static directory and preparing web v1.0.2; preserve immutable v1.0.1 tag. Backend components remain v1.0.1. All actual release pipelines contain no deployment jobs.

API documentation tests hit a compiler-killed mediasoup C++ build with Ninja -j32. Configured non-secret MESON_ARGS=-Ddebug=false in GitLab and backend CI to reduce native compiler memory without disabling assertions or tests; production Dockerfiles unchanged. Retried failed job 7327000 as 7327216.

Media unit validation also hit a native compiler kill from a pre-setting job; retried as 7327272. API integration ran 423 tests with one flaky failure: mutating the final JWT base64 character can alter only unused bits. Corrected the test to mutate the first signature character, without changing production verification. Retried immutable-tag integration validation as 7327325; validating the corrected test locally against disposable Keycloak.
<!-- SECTION:NOTES:END -->
