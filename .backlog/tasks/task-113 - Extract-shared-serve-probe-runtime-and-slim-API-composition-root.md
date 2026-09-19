---
id: TASK-113
title: Extract shared serve/probe runtime and slim API composition root
status: Done
assignee: []
created_date: '2026-09-19 11:31'
updated_date: '2026-09-19 11:34'
labels:
  - backend
  - architecture
dependencies: []
modified_files:
  - backend/crates/telemetry/src/lib.rs
  - backend/bins/api/src/main.rs
  - backend/bins/api/src/cli.rs
  - backend/bins/api/src/settings.rs
  - backend/bins/api/src/migrate.rs
  - backend/bins/api/src/serve.rs
  - backend/bins/api/Cargo.toml
  - backend/bins/media/src/main.rs
  - backend/bins/media/Cargo.toml
  - backend/bins/worker/src/main.rs
  - backend/bins/worker/Cargo.toml
priority: medium
ordinal: 121000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The three runtime binaries duplicate HTTP listener serving, shutdown waiting, and periodic dependency probes. mahfel-api's composition root also mixes CLI parsing, settings mapping, migration, listener wiring, and probe logic in one main.rs. Extract genuinely shared runtime helpers into mahfel-telemetry and split the API binary into small composition modules so composition roots stay thin and infrastructure wiring is not duplicated. No behavior change expected.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 mahfel-telemetry exposes reusable serve_http, wait_for_shutdown, and a generic periodic dependency probe runner
- [x] #2 mahfel-api, mahfel-media, and mahfel-worker use the shared helpers and no longer define their own wait_for_shutdown
- [x] #3 mahfel-api main.rs is a thin dispatch root with cli/settings/migrate/serve modules
- [x] #4 mahfel-postgres gains no dependency on mahfel-telemetry
- [x] #5 cargo fmt, clippy, and workspace tests pass; probes still update readiness
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add serve_http + wait_for_shutdown + spawn_probe to mahfel-telemetry (tokio watch based, generic check closure). 2. Split bins/api/src into cli.rs, settings.rs, migrate.rs, serve.rs; main.rs dispatch only. 3. Rewire media and worker to shared helpers. 4. fmt/clippy/test; verify readiness probes still wired.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented: mahfel-telemetry now exposes serve_http(listener, router, shutdown), wait_for_shutdown(shutdown), and spawn_probe(name, interval, health, shutdown, check) where the runner owns interval/shutdown/health wiring and the closure owns dependency-specific probing + metrics. Removed duplicated wait_for_shutdown from all three binaries; api/media/worker now call the shared helpers. Split bins/api/src into main.rs (dispatch only), cli.rs (Command parse), settings.rs (pool/connection mapping), migrate.rs, serve.rs (composition + listeners + postgres probe). Removed now-unused axum dep from media and axum/lapin deps from worker (lapin Connection is not Clone, worker wraps it in Arc for the probe closure). mahfel-postgres untouched and still has no telemetry dependency. Added telemetry test probe_runner_degrades_readiness_and_stops_on_shutdown covering failed-check readiness and shutdown stop.

Validation: cargo fmt --check clean; cargo clippy --workspace --all-targets --all-features -D warnings clean; cargo test --workspace --all-features --lib --bins passed (telemetry 2, api 14, postgres 2, domain 7); just docs strict build passed earlier for the path change.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted shared runtime helpers (serve_http, wait_for_shutdown, generic spawn_probe) into mahfel-telemetry and slimmed mahfel-api's composition root into cli/settings/migrate/serve modules. Removed duplicated shutdown/probe code from api, media, and worker without changing behavior. Verified with workspace fmt/clippy, workspace unit tests, and a new telemetry probe-runner test.
<!-- SECTION:FINAL_SUMMARY:END -->
