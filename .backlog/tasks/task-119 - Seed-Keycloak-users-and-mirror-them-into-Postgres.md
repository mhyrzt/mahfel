---
id: TASK-119
title: Seed Keycloak users and mirror them into Postgres
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
updated_date: '2026-09-23 22:09'
labels:
  - backend
  - auth
  - seed
milestone: m-2
dependencies:
  - TASK-9
  - TASK-117
  - TASK-12
priority: high
ordinal: 127000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a Keycloak Admin adapter that authenticates with the client-credentials grant of the mahfel-admin service account (no interactive administrator credentials), and make mahfel-api seed upsert the development identities into Keycloak and then mirror users and identities into Postgres. Add a nullable users.avatar_url column (S3 link) with no S3 client yet.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The Keycloak admin adapter authenticates with client credentials and is the only path mahfel-api uses to manage the realm
- [x] #2 seed creates missing development users in Keycloak and leaves existing users untouched; it never resets an existing password
- [x] #3 seed mirrors every fixture user into users and identities idempotently, including users that already existed in Keycloak
- [x] #4 A nullable users.avatar_url column exists via a forward migration and is returned by the profile endpoints
- [x] #5 The realm import no longer hardcodes development users; the seed is the single source of fixture identities
- [x] #6 Integration tests run against a disposable Keycloak and prove create-if-missing and mirror idempotency
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add required validated email and username fields to Mahfel users while retaining provider-owned identity email and username. 2. Add the client-credentials Keycloak admin operations for exact username lookup and create-if-missing; preserve existing user representations and passwords. 3. Make the canonical development-users fixture seed Keycloak and mirror each provider identity into its stable Mahfel user ID idempotently. 4. Keep development users out of the realm import and configure the app_user_id profile attribute as admin-only. 5. Add disposable Keycloak and PostgreSQL integration coverage; update local/Minikube configuration and docs; run focused checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added required, validated user email and username fields, persisted identity username, and rejected missing/malformed claims at provisioning. The canonical development-users fixture owns stable Mahfel IDs so restarting in-memory Keycloak does not duplicate local users. The seed creates absent Keycloak accounts, writes the app_user_id attribute only for new accounts, and mirrors existing/new accounts idempotently. Existing Keycloak profile fields and passwords are preserved; the user-profile schema makes app_user_id admin-only. Integration coverage: disposable Keycloak create-if-missing/attribute update (3 tests passed); disposable PostgreSQL migration, required-field constraints, mirror idempotency, and rotated-subject behavior (passed). Other checks: domain 88 tests, application 111 tests, API identity/profile tests, Clippy, Rust formatting, SQLFluff, Compose config, Minikube Kustomize rendering, strict docs build.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented idempotent Keycloak development-user creation and PostgreSQL mirroring with stable Mahfel IDs, required email/username fields, and admin-only app_user_id linkage. Existing Keycloak users and passwords remain untouched. Verified with disposable Keycloak and PostgreSQL integration tests plus backend lint, formatting, deployment configuration, and docs checks.
<!-- SECTION:FINAL_SUMMARY:END -->
