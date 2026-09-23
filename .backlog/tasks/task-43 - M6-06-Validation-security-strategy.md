---
id: TASK-43
title: M6-06 Validation/security strategy
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 01:13'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 51000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Client MIME and extensions are untrusted.

Implementation scope: Sniffing/quarantine decision, allowed types, malware hook, image normalization, quotas and observability.

Dependencies: M6-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Threat model documented; executable/polyglot test fixtures follow chosen policy.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Presentation uploads need explicit allowed PDF and raster-image formats, content sniffing, file-size and page or image-count limits, and any required image normalization. Do not trust client MIME types or extensions.
<!-- SECTION:NOTES:END -->
