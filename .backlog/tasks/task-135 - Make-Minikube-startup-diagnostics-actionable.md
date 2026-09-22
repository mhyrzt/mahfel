---
id: TASK-135
title: Make Minikube startup diagnostics actionable
status: Done
assignee: []
created_date: '2026-09-21 20:30'
updated_date: '2026-09-21 20:40'
labels: []
dependencies: []
ordinal: 143000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
First startup timed out waiting for Caddy after 180 seconds while the node was still pulling its image. Improve startup progress and report pod events/logs on failure.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Initial Caddy pull can exceed three minutes without failing the deployment
- [x] #2 A failed deployment or job prints an actionable pod reason and recent events/logs
- [x] #3 Minikube startup output clearly identifies each phase and the final next step
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Extend first-start rollout wait; add concise phase logging and targeted failure diagnostics; validate with shell checks and the live Minikube path.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Live evidence: Caddy image pull waited 11m22s before starting, exceeding the former 180s timeout. RustFS init also waited on an image pull. RabbitMQ readiness probe timed out at 1s; rabbitmq-diagnostics ping succeeded when run directly. Updated waits and diagnostics, increased probe timeout to 10s. The rerun reached all application rollouts; final host CA installation needs interactive sudo in this tool session. After restarting the worker following the RabbitMQ rollout, all 11 deployments were 1/1 and both jobs completed. Bash syntax, ShellCheck, strict docs build, and diff check passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Improved Minikube startup phase logging and targeted failure diagnostics, allowed 15m first image pulls for Caddy and RustFS init, and fixed RabbitMQ readiness probe timeout. Live Minikube deployments and jobs are healthy; host CA trust remains an interactive sudo step.
<!-- SECTION:FINAL_SUMMARY:END -->
