---
id: TASK-56
title: M9-01 Recording architecture spike
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
ordinal: 64000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: SFU recording format/topology determines reliability and cost.

Implementation scope: Evaluate mediasoup plain transports/taps, per-track versus composite, RTP capture, ffmpeg/GStreamer, node-local buffering and failure model.

Dependencies: M3, M6, M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ADR selects strategy with measured prototype, resource profile, restart limitations and alternatives.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Evaluate collaboration replay alongside A/V capture: timestamped whiteboard and text edits, snapshots, shared viewport changes, and references to stored presentation assets plus presenter slide changes. Interactive reconstruction is the baseline; a downloadable composited video requires a separate render/encode path and is optional.

Design clarification: The recording renderer may use a dedicated composition and layout independent of the live frontend. Evaluate this as a separate renderer over canonical media and collaboration timelines, not as a screenshot or clone of the live UI.
<!-- SECTION:NOTES:END -->
