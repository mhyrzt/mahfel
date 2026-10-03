---
id: TASK-177
title: Review Hamravesh networking requirements against official docs
status: Done
assignee:
  - '@codex'
created_date: '2026-10-03 07:19'
updated_date: '2026-10-03 07:24'
labels: []
dependencies: []
ordinal: 185000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Revise TO_HAMRAVESH.md using current official Hamravesh and Darkube documentation: remove answered generic questions, retain deployment-specific networking gaps and add missing constraints.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Documented platform capabilities are cited and separated from unresolved requirements
- [x] #2 Remaining questions cover media and TURN routing, port preservation, relay ranges and deployment constraints without GitOps content
- [x] #3 Embedded Compose reference remains valid and documentation diff passes whitespace checks
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Read official networking and Darkube settings docs; reconcile requirements; validate the resulting document and embedded Compose example.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reviewed official network IP, HTTP load balancer, Darkube ports/CustomConfig/domain/FAQ, dedicated Kubernetes access/components/storage and monitoring docs. Shared Darkube fixed-port reservation restriction is now a specific compatibility question; TCP/UDP definitions, internal networking, probes/lifecycle and basic metrics are documented capabilities.

Validation passed: embedded Compose config with disposable fixtures, no table/GitOps regressions, git diff --check and strict MkDocs build. Official docs read through browser tools and direct public HTTP where browser extraction failed. Incorporated user confirmation that a dedicated cluster IP has already been purchased; its service type and forwarding configuration remain uninspected. No cluster changes or live networking trial.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Updated TO_HAMRAVESH.md with cited documented capabilities, removed generic answered questions and grouped remaining media-specific gaps. Highlighted shared Darkube fixed-public-port restrictions and clarified dedicated outbound/domain inbound services versus the existing purchased allocation. Added egress/relay address consistency, product topology, maintenance, protected TURN TLS mounts, scratch-space limits and traffic accounting questions. Preserved ASCII diagram and Compose reference. Local documentation and Compose checks passed; platform/account networking remains unverified.
<!-- SECTION:FINAL_SUMMARY:END -->
