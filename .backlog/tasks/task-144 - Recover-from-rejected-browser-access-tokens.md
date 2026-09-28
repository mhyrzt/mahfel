---
id: TASK-144
title: Recover from rejected browser access tokens
status: Done
assignee:
  - '@codex'
created_date: '2026-09-26 11:23'
updated_date: '2026-09-26 11:45'
labels: []
dependencies: []
ordinal: 152000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A stale or rejected Keycloak access token currently produces a generic workspace error; the Next.js proxy can also redirect an errored session back to login repeatedly. Restore a clear sign-in path without exposing tokens.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 An API 401 on an authenticated page leads to the sign-in flow instead of the generic workspace error
- [x] #2 A session with a terminal error can load the login and NextAuth endpoints without a redirect loop
- [x] #3 404 room and session responses still reach the not-found route
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Classify upstream API 401 as an authentication failure without exposing Axios request headers. 2. Keep login and NextAuth routes accessible when a stored session has an error. 3. Preserve 404 routing through a typed sanitized API error. 4. Validate frontend gates and the running login flow.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Frontend lint, typecheck, formatting, Storybook build, and Docker production build passed. The local Keycloak login probe authenticated, loaded workspace and room pages, and received 200 from the backend auth endpoint. Rejected-token and 404 branches were verified by source inspection; they were not exercised in the live probe.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Classified API 401 responses as authentication failures, kept login and NextAuth routes accessible to errored sessions, and preserved room/session 404 routing through a sanitized status-bearing error.
<!-- SECTION:FINAL_SUMMARY:END -->
