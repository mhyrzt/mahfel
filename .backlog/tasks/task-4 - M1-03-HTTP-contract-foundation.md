---
id: TASK-4
title: M1-03 HTTP contract foundation
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 11:22'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
modified_files:
  - backend/AGENTS.md
  - backend/config/default.toml
  - backend/config/local.toml.example
  - deploy/dex/config.yaml
  - .env.example
  - docs/deployment/local.md
  - docs/backlog/roadmap.md
priority: medium
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Clients need stable errors, pagination, validation, and versions.

Implementation scope: /api/v1, request IDs, JSON error envelope, validation/extractors, pagination convention, CORS/security middleware.

Dependencies: M0, M1-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Contract tests cover success/error/request ID; invalid input never becomes generic 500.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect current api main.rs and workspace http needs 2. Create bins/api/src/http module: error envelope (ApiError + JSON envelope + request_id), validated Json/Query extractors, pagination params, security headers 3. Wire router under /api/v1 with middleware stack: CORS (explicit methods/headers), request-id propagate/set, trace, security headers, error handling 4. Add contract tests: success, validation error (422 not 500), request-id propagation, pagination defaults/bounds, CORS/security headers
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented versioned /api/v1 router in bins/api/src/http: typed error envelope (ErrorCode + ApiError) rendered as {"error":{code,message,details?}}, ValidatedJson/ValidatedQuery/ValidatedPagination extractors mapping malformed JSON to 422 and bad query/pagination to 400 (never 500), Typed PaginationError enum instead of stringly errors, offset pagination (page>=1, per_page 1..=100) with data+pagination response shape, exact-match CORS allowlist exposing x-request-id, request-id set/propagate/trace layers, and security headers (nosniff, DENY, referrer-policy, permissions-policy). Unknown routes return JSON 404 envelope via top-level fallback. main.rs now composes api_router + cors_layer; removed ad-hoc status route. Documented the HTTP contract in docs/development/conventions.md.

Validation: cargo fmt --all --check clean; cargo clippy --workspace --all-targets --all-features -D warnings clean; cargo test -p mahfel-api 13 passed; cargo test --workspace --all-features --lib --bins passed; just docs strict build passed. Docker-backed integration tests not required for this task.

Refactor after review: split bins/api/src/http/mod.rs into single-concern modules — mod.rs (composition only), cors.rs, middleware.rs (security headers), request_id.rs, error.rs, extractor.rs, pagination.rs, routes/{mod,status,health,echo}.rs, and tests.rs. Added an unversioned-unknown-route 404 contract test (14 tests total). No behavior change.

Path change after review: API version prefix is /v1 (not /api/v1) because mahfel-api is served on its own host (api.mahfel.localhost; darkube routes web and API to separate ingresses), so /api was redundant. Updated router nest, http module docs/comments, tests, backend/AGENTS.md, docs/development/conventions.md, docs/deployment/local.md, docs/backlog/roadmap.md, and OIDC redirect fixtures (backend/config/default.toml, backend/config/local.toml.example, deploy/dex/config.yaml, .env.example). 14 tests pass; just docs strict build passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Established the /api/v1 HTTP contract foundation: stable JSON error envelope with never-500 bad input handling, request-id correlation, validated JSON/query/pagination extractors with a typed pagination error, offset pagination convention, exact-match CORS, and baseline security headers. Verified with workspace clippy/fmt, 13 API contract tests, workspace unit tests, and a strict docs build.
<!-- SECTION:FINAL_SUMMARY:END -->
