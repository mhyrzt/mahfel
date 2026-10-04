---
id: TASK-179
title: Integrate Mahfel production deployment with Rastploy
status: Done
assignee:
  - '@codex'
created_date: '2026-10-03 11:52'
updated_date: '2026-10-03 12:04'
labels: []
dependencies: []
ordinal: 187000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define production Compose and Rastploy targets in the DevOps submodule, a rastai.ai DNS zone, shared PostgreSQL/RabbitMQ provisioning and protected component release CD inspired by LMS. Keep external Keycloak/S3, independent application releases and private operations ports. Prepare and validate configuration without deploying or committing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 DevOps has independently versioned Mahfel production targets and public DNS/Caddy routing with private control ports
- [x] #2 Shared PostgreSQL and RabbitMQ provisioning isolates Mahfel state and applies quorum dead-letter policy
- [x] #3 Protected component release CI deploys the published version through scoped HTTPS Rastploy access and waits for success
- [x] #4 Environment references, manifests, Compose, DNS, shell, CI and docs validate with disposable fixtures
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Follow DevOps service conventions and runtime contracts. 2. Add app/dependency targets, shared provisioning, DNS and Caddy routes. 3. Adapt LMS HTTPS release deployment with terminal status checks. 4. Document secrets, first deployment and external prerequisites; validate locally without live changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added six independent production Compose/Rastploy targets in devops/services/mahfel, private and external network wiring, migration-before-API ordering, bounded resources, direct RTC and host-network TURN, external identity/object storage templates, shared PostgreSQL role/database and isolated RabbitMQ vhost/user/policy provisioning. Added rastai.ai zone and Caddy routes with public internal-render denial. Adapted LMS HTTPS CD to protected component releases, blocking stage ordering, exact published image tags, scoped token access and terminal polling. Validation: current-source Rastploy reports 36 targets; disposable full-tree environment generation/check passes; six Mahfel plus PostgreSQL/RabbitMQ Compose configs validate; rendered DNS zone passes named-checkzone; Caddy 2.11.4 adapt/validate passes in a no-network container; shell syntax/ShellCheck, eight CI includes/rules/stages, six deployment script tests, strict docs and both repo whitespace checks pass. RabbitMQ provisioning passes twice in an isolated container, preserves fixture admin and verifies permission/policy scope. Initial disposable broker startup failed due local anonymous-volume cookie permissions; private UID-owned tmpfs resolved it. PostgreSQL SQL provisioning was not executed because its image is not cached; no production infrastructure or credentials were touched. Production ciphertext, external client/bucket settings, VPS address, TURN certificates and GitLab token remain operator setup. No commits, pushes or deployments performed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Prepared production Mahfel manifests and DNS/Caddy integration in the DevOps submodule with shared PostgreSQL/RabbitMQ provisioning and independent protected-release Rastploy CD. Focused source/configuration, shell, DNS, Caddy, disposable RabbitMQ and CI tests passed. Real secrets, external prerequisites, PostgreSQL execution and live application/network acceptance remain deployment preparation; changes are uncommitted.
<!-- SECTION:FINAL_SUMMARY:END -->
