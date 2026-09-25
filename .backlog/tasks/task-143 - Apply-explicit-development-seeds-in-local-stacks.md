---
id: TASK-143
title: Apply explicit development seeds in local stacks
status: Done
assignee: []
created_date: '2026-09-25 19:29'
updated_date: '2026-09-25 20:01'
labels:
  - deployment
  - backend
dependencies: []
ordinal: 151000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Compose and Minikube must apply backend/seeds TOML fixtures, including local file assets, in repeatable test deployments.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Compose seed service reads backend/seeds from the checkout, waits for PostgreSQL, Keycloak, and storage bucket, and uses matching storage credentials
- [x] #2 Minikube seed Job reads backend/seeds from the API image after storage initialization and reruns on up
- [x] #3 Local deployment docs describe seed update workflow and supported asset paths
- [x] #4 Compose config and Kustomize rendering pass; changed manifests format, Minikube script parses, backend fmt/lint/unit tests pass, and unavailable network-dependent image/schema checks are recorded
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Mount or package the explicit seed directory. 2. Fix dependency order and rerun behavior. 3. Validate deployment configuration and seed tests. 4. Update docs and split commits.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Compose reads checkout seeds through a bind mount; Minikube API image packages the seed directory, and up recreates the seed Job after RustFS initialization.

Validation: docker compose config, kubectl kustomize, changed YAML and docs formatting, bash -n, backend fmt/lint, all 379 backend unit tests, and strict docs build passed. Full Minikube prettier gate reports 15 pre-existing unrelated YAML files in addition to the changed seed manifest; changed manifest is formatted. kubeconform schema fetch is blocked by local proxy connection refusal. API image build reaches cargo-chef but cannot download the pinned Rust channel metadata from static.rust-lang.org, so a built image and live Minikube run are not verified.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Compose now mounts backend/seeds and waits for PostgreSQL, Keycloak, and RustFS initialization with matching S3 settings. Minikube packages seeds in the API image and recreates the seed Job after storage setup on each up. Docs explain reseeding and container asset paths. Static configuration and backend checks pass; image build and schema lint could not complete because network access is unavailable.
<!-- SECTION:FINAL_SUMMARY:END -->
