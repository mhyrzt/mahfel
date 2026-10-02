---
id: TASK-169
title: Publish v1.0.1 images for all four apps
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-02 09:21'
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
- [ ] #1 All four app versions and release tags identify v1.0.1 and the source includes the CI repairs
- [ ] #2 Protected component release pipelines publish all four registry images with recorded digests
- [ ] #3 Release pipelines do not create deployment jobs in publish-only mode
- [ ] #4 Web image uses the approved production API URL and all relevant validation gates pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add an explicit publish-only pipeline switch and validate release rules. 2. Set versions to 1.0.1 and regenerate API contracts. 3. Configure tag protection and required build variables. 4. Push component tags with deployment disabled and track builds/publishing to completion. 5. Verify registry digests and finalize release notes.
<!-- SECTION:PLAN:END -->
