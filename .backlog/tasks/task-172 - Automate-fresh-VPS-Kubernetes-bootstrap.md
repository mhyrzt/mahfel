---
id: TASK-172
title: Automate fresh VPS Kubernetes bootstrap
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 13:28'
updated_date: '2026-10-02 14:35'
labels: []
dependencies: []
ordinal: 180000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a reviewed script for a fresh Ubuntu 24.04 LTS x86-64 systemd VPS: OS prerequisites, firewall, pinned K3s/Argo CD, Sealed Secrets bootstrap and public certificate export. Application deployment waits for configured external services and encrypted secrets. Logs are readable, concise, timed and keep credentials out of console output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Plan mode makes no host or cluster changes and validates explicit bootstrap inputs
- [x] #2 Apply mode prepares fresh host networking/firewall and pinned K3s, Argo CD and Sealed Secrets without deploying unconfigured applications
- [x] #3 Repeat runs preserve cluster keys/state and reject incompatible existing configuration or versions
- [x] #4 Progress logging is concise with color/NO_COLOR, durations and protected detailed failure logs
- [x] #5 Script behavior, shell syntax and runbook instructions validate; real VPS bootstrap remains an explicit verification boundary
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Implement plan/apply CLI and Ubuntu 24.04 preflight guards. 2. Add timed progress and protected step logs. 3. Automate prerequisites/firewall, pinned K3s and controller bootstrap. 4. Verify CLI, isolated logging/configuration/key-backup failure paths, shell checks and docs; qualify actual host networking on the VPS later.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented deploy/k3s/bootstrap-vps.sh with required public IPv4/admin CIDR/exact version inputs, Ubuntu 24.04-only guard, SSH checks, compatible rerun guards, UFW/external firewall choice, private Argo CD and Sealed Secrets bootstrap. Console logs have six timed phases, terminal color/NO_COLOR, 20-second heartbeats and sanitized failure line plus protected log path. Recovery outputs are private and published after successful retrieval/parsing. Validation: bash syntax, ShellCheck, seven Python tests with invalid-input subcases and isolated shell function stubs, strict MkDocs build, deployment document links and git diff whitespace check passed. No apply was run on a real VPS; package installation, firewall rules, controller sync, ingress/RTC/TURN connectivity and live acceptance remain deployment verification.

Relocated bootstrap to deploy/vps/bootstrap.sh at user request because it orchestrates host preparation and multiple cluster components. Moved its documentation/tests alongside it, kept K3s-specific manifests in deploy/k3s, and updated runbook references and test entry point. Shell syntax, ShellCheck, seven tests, strict docs, document links, executable permission and whitespace checks passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Prepared Ubuntu 24.04 VPS bootstrap and runbook with concise timed logs, protected diagnostics, exact release inputs, guarded reruns and controller key backup. Local behavior and documentation checks passed; actual VPS installation/network qualification remains explicitly unverified.
<!-- SECTION:FINAL_SUMMARY:END -->
