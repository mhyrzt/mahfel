---
id: TASK-143
title: Apply explicit development seeds in local stacks
status: Done
assignee: []
created_date: '2026-09-25 19:29'
updated_date: '2026-09-25 20:53'
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
- [x] #5 Minikube bootstrap initializes the profile and kubectl context before building images
- [x] #6 Bootstrap initializes the Minikube context first, and the container builder installs all pinned Rust components
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Mount or package the explicit seed directory. 2. Fix dependency order and rerun behavior. 3. Validate deployment configuration and seed tests. 4. Update docs and split commits.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Compose reads checkout seeds through a bind mount; Minikube API image packages the seed directory, and up recreates the seed Job after RustFS initialization.

Validation: docker compose config, kubectl kustomize, changed YAML and docs formatting, bash -n, backend fmt/lint, all 379 backend unit tests, and strict docs build passed. Full Minikube prettier gate reports 15 pre-existing unrelated YAML files in addition to the changed seed manifest; changed manifest is formatted. kubeconform schema fetch is blocked by local proxy connection refusal. API image build reaches cargo-chef but cannot download the pinned Rust channel metadata from static.rust-lang.org, so a built image and live Minikube run are not verified.

Follow-up after user startup attempt: bootstrap now runs Minikube init before build, and the container builder requests clippy as well as rustfmt per the pinned toolchain. Dry-run confirms command order and Compose config/docs checks pass. API image build reaches clippy artifact download, which remains blocked by connection refusal to static.rust-lang.org.

Documented the Rust distribution network requirement. Follow-up image build confirms clippy is the missing pinned toolchain component; downloading it is still unavailable due connection refusal. Bootstrap dry-run order, Compose config, docs build, and changed-file format checks pass.

Follow-up after local failures: bootstrap now initializes Minikube and selects its context before build. Compose and Minikube image builds use host networking; just compose-up and the Minikube builder forward configured proxy environment variables to BuildKit. Compose config, Minikube bootstrap dry-run order, script syntax, docs formatting, and diff checks pass. Full image build remains unverified because downloads through the host proxy are exceptionally slow; Debian packages advanced at about 25 kB/s before validation was stopped.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Local stack seed wiring is complete. Bootstrap initializes Minikube before image builds, and container builds explicitly install the pinned Clippy and rustfmt components. Compose/Kustomize configuration, docs, backend tests, and lint pass; a full image build requires network access to static.rust-lang.org.

Bootstrap now initializes Minikube before building. Compose and Minikube build paths use host networking and forward configured proxy variables without hardcoding proxy values. Configuration and static checks pass; full image build needs a faster reachable package/Rust mirror or network.
<!-- SECTION:FINAL_SUMMARY:END -->
