---
id: TASK-61
title: M9-06 Playback and retention
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
updated_date: '2026-09-23 01:21'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 69000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Finished recordings need authorized access and lifecycle management.

Implementation scope: Recording API/UI, presigned playback, duration/status, retention/delete policy and audit.

Dependencies: M9-05, M6-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Authorized playback works; unauthorized URL cannot be obtained; retention deletion is tested.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Playback loads the required snapshot and stored presentation assets, then applies timestamped collaboration and shared-viewport events in order alongside A/V. All participants reconstruct the same room view; downloadable video export is a separate optional capability.

Design clarification: Recording playback/export may present reconstructed canvas, document, slides, and A/V in a purpose-built recording layout independent of the live frontend. Keep that presentation layer separate from event/state reconstruction.
<!-- SECTION:NOTES:END -->
