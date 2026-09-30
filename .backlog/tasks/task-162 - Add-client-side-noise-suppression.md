---
id: TASK-162
title: Add client-side noise suppression
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
ordinal: 170000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Let participants enable or disable microphone noise suppression before their audio is published to WebRTC. Start with supported browser capture constraints; evaluate additional local processing only if browser filtering cannot meet the agreed quality target. Keep processing on the sender so recipients and recordings receive the same processed audio. Expose supported state truthfully and retain working microphone capture on unsupported browsers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Participants can toggle noise suppression with localized English/Persian controls and accessible state; supported settings apply to the published microphone track.
- [ ] #2 Unsupported browsers or processing failures show an explicit fallback and do not prevent microphone use or silently claim suppression is active.
- [ ] #3 Microphone switching, mute/unmute, reconnect and leaving clean up or reapply processing without duplicate tracks or leaked resources.
- [ ] #4 Component/Storybook coverage and browser audio checks verify the toggle, fallback and processed outgoing audio; document supported browser behavior and quality limits.
<!-- AC:END -->
