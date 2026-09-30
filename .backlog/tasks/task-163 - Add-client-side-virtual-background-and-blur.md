---
id: TASK-163
title: Add client-side virtual background and blur
status: To Do
assignee: []
created_date: '2026-09-29 09:17'
labels:
  - frontend
  - media
dependencies: []
references:
  - frontend/features/sessions/components/use-session-media.ts
priority: medium
ordinal: 171000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Let participants blur their camera background or replace it with a selected image using local person segmentation and compositing before WebRTC publication. Publish the processed camera track and show the same output in local preview so recipients and recordings see the chosen effect. Include browser/device capability detection and a usable fallback; preserve the current media-service boundaries.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Participants can select no effect, background blur or an image background through localized English/Persian and keyboard-accessible controls.
- [ ] #2 The outgoing camera track and local preview both show the selected effect; remote participants and recording capture receive the processed video rather than a preview-only CSS effect.
- [ ] #3 Unsupported devices or processing failures show an explicit fallback; measured CPU/GPU, frame rate and latency limits inform degradation and are documented.
- [ ] #4 Camera switching, effect changes, mute/unmute, reconnect and leaving correctly manage raw/processed tracks and release processing resources; Storybook, component and real-browser checks cover the supported flows.
<!-- AC:END -->
