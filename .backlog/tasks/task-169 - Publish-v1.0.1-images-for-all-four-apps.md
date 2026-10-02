---
id: TASK-169
title: Publish corrected release images for all apps
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-02 09:21'
updated_date: '2026-10-02 14:53'
labels:
  - release
  - ci
dependencies: []
ordinal: 177000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish corrected API, media and worker v1.0.1 images and web v1.0.2. Preserve existing release tags, advisory security scans and blocking validation/build gates. Publish images and GitLab releases only; do not execute the former deployment platform deployment. Web production URL is https://mahfel.rastai.ai and its approved API build URL is https://mahfel-api.rastai.ai.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Release pipelines do not create deployment jobs in publish-only mode
- [x] #2 Web image uses the approved production API URL and all relevant validation gates pass
- [x] #3 API, media, and worker versions and tags are v1.0.1; corrected web version and tag are v1.0.2, including all CI repairs.
- [ ] #4 All four images from successful protected release builds are published under hamdars/mahfel-component with verified registry and image configuration digests.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Change CI image repository composition to MAHFEL_IMAGE_PREFIX-component and update documentation. 2. Verify all protected release gates passed and locate their exact image archives. 3. Publish those validated archives under mahfel-api, mahfel-media, mahfel-worker, and mahfel-web without rebuilding or moving tags. 4. Verify registry manifests and image configuration digests against the archives. 5. Update GitLab releases and finalize tracking; no deployment.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Web v1.0.1 production and Docker builds passed, but Storybook failed because staticDirs still referenced removed recording-renderer-dist. Removed the obsolete Storybook static directory and preparing web v1.0.2; preserve immutable v1.0.1 tag. Backend components remain v1.0.1. All actual release pipelines contain no deployment jobs.

API documentation tests hit a compiler-killed mediasoup C++ build with Ninja -j32. Configured non-secret MESON_ARGS=-Ddebug=false in GitLab and backend CI to reduce native compiler memory without disabling assertions or tests; production Dockerfiles unchanged. Retried failed job 7327000 as 7327216.

Media unit validation also hit a native compiler kill from a pre-setting job; retried as 7327272. API integration ran 423 tests with one flaky failure: mutating the final JWT base64 character can alter only unused bits. Corrected the test to mutate the first signature character, without changing production verification. Retried immutable-tag integration validation as 7327325; validating the corrected test locally against disposable Keycloak.

Web v1.0.2 passed all blocking checks and Docker build, but publish job 7327157 failed after successful registry login: account aiedut is unauthorized to upload to registry/hamdars/mahfel/web/_uploads. Existing Docker auth uses the same account. Requested registry write permission or project upload credentials; no alternative namespace was used and no deployment occurred. Worker all tests and Docker build passed; remaining backend retries continue.

Corrected JWT tampering integration test passed locally against disposable Keycloak, and workspace formatting passed. API documentation retry 7327216 passed. Registry upload permissions remain the publication blocker for completed worker and web builds.

Final current state: worker pipeline 2116231, media 2116230, and corrected web 2116264 passed all blocking tests/builds but publication was denied by registry upload permissions. API pipeline 2116229 still runs integration retry 7327325; all its other validation gates passed. GitLab release notes explicitly identify unpublished images and disabled deployment. Publication AC remains unchecked and task remains In Progress until write access is granted and all four registry digests are verified.

User requested flat mahfel-component registry repositories instead of mahfel/component. CI publisher and cleanup now use the flat names. All blocking validation/build jobs in the four protected release pipelines passed. Original publisher jobs contain the previous naming configuration, so publish their exact successful build artifacts directly under the newly approved repository names, without rebuilding or moving tags.
<!-- SECTION:NOTES:END -->
