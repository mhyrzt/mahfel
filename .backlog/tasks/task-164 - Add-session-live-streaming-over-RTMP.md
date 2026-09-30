---
id: TASK-164
title: Add session live streaming over RTMP
status: To Do
assignee: []
created_date: '2026-09-29 09:17'
labels:
  - backend
  - frontend
  - media
dependencies: []
documentation:
  - docs/architecture/components.md
  - docs/adr/0004-use-mediasoup.md
priority: medium
ordinal: 172000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow authorized session hosts to publish a live audio/video broadcast to an external RTMP/RTMPS destination. Define the live output composition for selected speakers, screen sharing and any supported presentation content, encode it and bridge it from the media plane to the destination. Provide host controls and participant-visible status, manage failures independently of the interactive session, and document the architecture within existing service boundaries. This is a separate live-output feature, not an extension of M9 original-track recording or downloadable video generation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 An architecture/design decision defines supported output composition, codecs, destination protocol, latency/resource budgets and lifecycle ownership without routing media through the API or Next.js.
- [ ] #2 Authorized hosts can configure a destination and start/stop streaming; unauthorized commands fail, and participants see accurate localized starting/live/stopping/failed status.
- [ ] #3 Destination credentials and stream keys remain protected, are never exposed to other participants or logged, and destination validation restricts access to unintended internal endpoints.
- [ ] #4 Destination disconnects, retries, session end and service shutdown stop or recover the broadcast predictably without ending the interactive session or leaking capture/encoder resources.
- [ ] #5 A real RTMP/RTMPS receiver validates audio/video output and failure handling; document supported destinations, operational limits and required configuration, with UI/Storybook coverage for controls and status.
<!-- AC:END -->
