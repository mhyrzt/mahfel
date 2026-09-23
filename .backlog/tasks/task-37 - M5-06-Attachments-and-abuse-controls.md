---
id: TASK-37
title: M5-06 Attachments and abuse controls
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 14:58'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 45000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Chat files and bursts need safe limits.

Implementation scope: Attachment metadata link, pending/ready states, rate/size/content limits, moderation hooks.

Dependencies: M6-03, M5-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Only authorized ready objects attach; flood and invalid object tests fail predictably.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reuse the existing chat-message file context and ready-only listing. 2. Enforce a bounded per-message attachment count and live-message check atomically at metadata insertion. 3. Add shared Redis-backed message burst limits and explicit content limits. 4. Test pending versus ready visibility, invalid/deleted message attachment attempts, flood limits, and authorization.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Ready-only chat attachments, four-file limit, live-member/message checks, cleanup on moderation, content bounds, and Redis fixed-window posting limits. PostgreSQL attachment, Redis limiter, API 429, and RustFS tests passed in full integration suite.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented chat attachment controls and abuse limits with storage and rate-limit tests.
<!-- SECTION:FINAL_SUMMARY:END -->
