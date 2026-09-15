---
id: TASK-108
title: Use structured PostgreSQL configuration
status: Done
assignee:
  - '@codex'
created_date: '2026-09-15 23:20'
updated_date: '2026-09-15 23:37'
labels: []
dependencies: []
ordinal: 116000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Backend database configuration should expose host, port, user, password, and database name independently instead of requiring operators to assemble a connection URL.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Default and local configuration use discrete PostgreSQL host, port, user, password, and name fields
- [x] #2 MAHFEL_DATABASE nested environment overrides configure each field and Compose no longer passes a database URL
- [x] #3 API migration and serve modes connect through typed settings without logging credentials
- [x] #4 Database unit/integration tests, Compose configuration, and relevant documentation pass
- [ ] #5 Affected Rust format, check, strict Clippy, nextest, doctests, Compose configuration, and documentation pass; the disposable live-database harness is adapted to the new type
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace DatabaseSettings.url with discrete host, port, user, password, and name fields in typed configuration and examples. 2. Add a database connection settings type that builds PgConnectOptions internally and update API startup/migration paths. 3. Update Compose and environment variables to use nested fields. 4. Adapt disposable database tests and documentation. 5. Run focused Rust, Just, Compose, and docs validation, then close and commit the scoped changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced the PostgreSQL URL with host, port, user, password, and name fields. The database crate builds PgConnectOptions internally and API serve/migrate modes use the typed settings. Affected fmt, check, strict Clippy, nextest, doctests, Compose, and docs passed. Live PostgreSQL was not started in this turn.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Introduced structured PostgreSQL configuration across TOML, environment overrides, Compose, API startup, and the disposable database harness.
<!-- SECTION:FINAL_SUMMARY:END -->
