---
id: TASK-43
title: M6-06 Validation/security strategy
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 12:34'
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
- [x] #1 Threat model documented; executable/polyglot test fixtures follow chosen policy.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Document the threat model and allowlist; enforce size, checksum, MIME and format gates with owner quota and polyglot fixtures.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Presentation uploads need explicit allowed PDF and raster-image formats, content sniffing, file-size and page or image-count limits, and any required image normalization. Do not trust client MIME types or extensions.

Application tests passed executable and appended-polyglot fixtures; docs strict build passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Documented the upload threat model and enforced PDF/PNG/JPEG type gates, checksum and quota checks, and quarantine before ready.
<!-- SECTION:FINAL_SUMMARY:END -->
