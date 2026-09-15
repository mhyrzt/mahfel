---
id: TASK-109
title: Add trusted local HTTPS with Caddy
status: Done
assignee:
  - '@codex'
created_date: '2026-09-15 23:22'
updated_date: '2026-09-15 23:37'
labels: []
dependencies: []
ordinal: 117000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Local development must use HTTPS through Caddy, with the generated local CA explicitly installed into the developer host trust store so browser and API behavior matches secure production assumptions.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Compose includes a pinned Caddy service with persistent PKI state and loopback-only HTTP/HTTPS listeners
- [x] #2 Mahfel web and API are reachable through documented HTTPS .localhost hostnames and frontend/API origin configuration uses those URLs
- [x] #3 A cross-platform just recipe exports and installs Caddy's local root CA into supported Linux or macOS trust stores
- [x] #4 Documentation treats CA trust as a required setup step and Compose/Caddy static validation passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a Caddyfile for web and API .localhost reverse proxies with internal TLS. 2. Add a pinned Caddy Compose service, persistent data/config volumes, and local-only 80/443 bindings. 3. Update frontend API URL and API allowed origin defaults for HTTPS. 4. Add a root just recipe that exports the generated root CA and installs it on Fedora/RHEL, Debian/Ubuntu, or macOS. 5. Document the mandatory trust step and validate recipes, Caddy formatting/static config, Compose, and docs without pulling images.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added Caddy 2.11.4 through docker.arvancloud.ir with persistent PKI, trusted HTTPS routes for web, API, RustFS S3, and RustFS console, plus mandatory Fedora/RHEL, Debian/Ubuntu, and macOS trust installation. Official Caddy binary formatting/static validation and Compose config passed. CA installation was not executed because containers were intentionally not started.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added a trusted local HTTPS edge with persistent Caddy CA state, mandatory host trust workflow, and secure browser routes for Mahfel and RustFS.
<!-- SECTION:FINAL_SUMMARY:END -->
