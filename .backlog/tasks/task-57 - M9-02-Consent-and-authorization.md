---
id: TASK-57
title: M9-02 Consent and authorization
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
updated_date: '2026-09-23 01:13'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 65000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Recording can violate privacy unless state and policy are explicit.

Implementation scope: Teacher capability, participant notice/consent policy, jurisdiction/config hooks, immutable audit events.

Dependencies: M2-07, M4
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unauthorized start fails; every participant sees state; start/stop/consent is audited.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Recording notice, consent, authorization, and audit policy must cover captured collaboration events and presentation playback references as well as A/V.
<!-- SECTION:NOTES:END -->
