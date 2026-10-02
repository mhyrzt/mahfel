---
id: TASK-169
title: Publish corrected release images for all apps
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-02 09:21'
updated_date: '2026-10-02 10:52'
labels:
  - release
  - ci
dependencies: []
ordinal: 177000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish corrected API, media and worker v1.0.1 images and web v1.0.2. Preserve existing release tags, advisory security scans and blocking validation/build gates. Publish images and GitLab releases only; do not execute Darkube deployment. Web production URL is https://mahfel.rastai.ai and its approved API build URL is https://mahfel-api.rastai.ai.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Protected component release pipelines publish all four registry images with recorded digests
- [x] #2 Release pipelines do not create deployment jobs in publish-only mode
- [x] #3 Web image uses the approved production API URL and all relevant validation gates pass
- [x] #4 API, media, and worker versions and tags are v1.0.1; corrected web version and tag are v1.0.2, including all CI repairs.
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

Web v1.0.2 passed all blocking checks and Docker build, but publish job 7327157 failed after successful registry login: account aiedut is unauthorized to upload to registry/hamdars/mahfel/web/_uploads. Existing Docker auth uses the same account. Requested registry write permission or project upload credentials; no alternative namespace was used and no deployment occurred. Worker all tests and Docker build passed; remaining backend retries continue.

Corrected JWT tampering integration test passed locally against disposable Keycloak, and workspace formatting passed. API documentation retry 7327216 passed. Registry upload permissions remain the publication blocker for completed worker and web builds.

Final current state: worker pipeline 2116231, media 2116230, and corrected web 2116264 passed all blocking tests/builds but publication was denied by registry upload permissions. API pipeline 2116229 still runs integration retry 7327325; all its other validation gates passed. GitLab release notes explicitly identify unpublished images and disabled deployment. Publication AC remains unchecked and task remains In Progress until write access is granted and all four registry digests are verified.
<!-- SECTION:NOTES:END -->
