---
id: TASK-16
title: M2-08 Avatar resolution
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 18:55'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: OIDC avatar URLs and custom uploads need one safe response strategy.

Implementation scope: Claim URL validation/proxy decision, custom avatar object metadata, precedence/fallback and cache policy.

Dependencies: M2-04, M6 design input
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Strategy documented; unsafe schemes rejected; endpoint contract tested.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Define one avatar policy: expose provider HTTPS URLs directly without proxying; reject unsafe or malformed schemes; prefer the user-owned avatar URL over the provider claim and return no avatar when neither is valid. 2. Align /auth/provision and /auth/me response behavior, using private no-store caching for profile responses. 3. Document how avatar file metadata is represented and how custom and provider avatars fall back. 4. Add domain validation and API contract tests, then run focused backend formatting, lint, and tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Provider avatar URLs must parse as absolute HTTPS URLs with a host and no embedded credentials. Invalid optional picture claims are discarded without failing sign-in. Provider images are returned directly; the API does not proxy or fetch them. Profile responses use private, no-store caching. Both provisioning and /auth/me prefer users.avatar_url over the validated provider picture; /auth/me reads the local user record only for this profile field, while authentication still builds the principal from claims. Custom avatar files use the actor-keyed avatar file context and remain in private object storage with PostgreSQL metadata. Validation: cargo fmt --all -- --check; cargo test -p mahfel-domain -p mahfel-application (198 passed); cargo test -p mahfel-api identity_ (6 passed); cargo clippy -p mahfel-domain -p mahfel-application -p mahfel-api --all-targets -- -D warnings; just docs.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented the M2 avatar policy across both profile endpoints: stored user avatar first, validated provider picture fallback, and no avatar for invalid or absent claims. Avatar URLs require absolute HTTPS without credentials; invalid optional claims do not block login, and provider images are not proxied. Added private no-store profile caching and documented avatar file metadata. Verified with 198 domain/application tests, 6 identity API tests, Clippy, formatting, and strict docs build.
<!-- SECTION:FINAL_SUMMARY:END -->
