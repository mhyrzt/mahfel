---
id: TASK-9
title: M2-01 Identity schema
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 19:58'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: One Mahfel user may have multiple login identities.

Implementation scope: Users, identities (issuer, subject), verified emails, avatar source, local credential table, uniqueness/migrations.

Dependencies: M1
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Constraint tests prevent identity takeover and allow multiple identities per user.
- [x] #2 First login for an unseen (issuer, subject) provisions a local user idempotently; concurrent first logins cannot create duplicates (unique (issuer, subject) enforced with a conflict-safe upsert)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Migration 0004: add users.avatar_url; create identities (unique (issuer, subject), user_id index, email/email_verified/display_name/avatar_url) and local_credentials tables.
2. Domain: add Issuer, Subject, EmailAddress, AvatarUrl value objects; add Identity aggregate and avatar_url on User; re-export.
3. Application: add an IdentityStore port returning ProvisionedIdentity; implement the real sync_identity use case (validate claims, derive a fallback display name, resolve-or-provision).
4. Postgres: PostgresIdentityStore with a conflict-safe transaction (select, else insert user + identity ON CONFLICT DO NOTHING, rollback and re-select on conflict); register in PostgresStores; persist users.avatar_url.
5. Tests: domain validation; integration covering multiple identities per user, identity-takeover rejection, and concurrent first-login convergence on one user.
6. Validation: cargo fmt/clippy/test for domain/application/postgres with a disposable PostgreSQL.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Just-in-time provisioning requirement (raised during M1-07 review): an identity may authenticate at the IdP (Dex/Keycloak/Zitadel) before it exists in mahfel.users. The identities table plus a subject-keyed, conflict-safe upsert must create the local user on first login. Identity is keyed on (issuer, subject); email must never be a linking key. The sync_identity application slice is currently a non-persisting placeholder and must be completed against this schema.

Migration 0004 adds users.avatar_url (nullable, bounded, S3 link) plus identities (unique (issuer, subject), user_id index) and local_credentials. Domain gains Issuer, Subject, EmailAddress, AvatarUrl value objects (http/https-only avatars, email requires local@domain), an Identity aggregate with non-erasing profile refresh, avatar_url on User, and ValidationError::InvalidFormat. Application: new IdentityStore port (IdentityProfile, ProvisionedIdentity) and a real sync_identity use case that validates claims, derives a bounded fallback display name, and resolves-or-provisions. Postgres: PostgresIdentityStore total-function upsert (select, else transactional user+identity insert with ON CONFLICT DO NOTHING, rollback and adopt on conflict) and avatar_url persistence in PostgresUserStore. Tests: domain unit tests for the value objects and refresh semantics; integration covers first-login provisioning, concurrent convergence on one user via tokio::join!, multiple identities per user, cross-issuer separation, repeated-login refresh, and that a shared verified email never links two identities to one user. Validation: workspace clippy clean; domain/application/postgres unit tests pass; migrations integration test passes against a disposable PostgreSQL (4 migrations, identity lifecycle assertions); workspace lib/bins tests green.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered the M2-01 identity schema: users.avatar_url, an identities table keyed by (issuer, subject) with verified-email metadata, and local_credentials. Identity value objects and an Identity aggregate live in mahfel-domain; an IdentityStore port and the real sync_identity use case live in mahfel-application; a conflict-safe PostgresIdentityStore provisions idempotently and converges under concurrency. Email is never a linking key, so an unverified or shared email cannot take over an account.
<!-- SECTION:FINAL_SUMMARY:END -->
