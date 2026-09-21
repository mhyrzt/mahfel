---
id: TASK-134
title: Add Minikube development deployment
status: Done
assignee: []
created_date: '2026-09-21 15:27'
updated_date: '2026-09-21 15:41'
labels: []
dependencies: []
ordinal: 142000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provide a local Kubernetes deployment for Mahfel application and required infrastructure, with a reproducible image build, browser access, and validation workflow.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Minikube manifests deploy the four application components and local dependencies with private control ports
- [x] #2 Developer documentation covers image builds, startup, browser access, media limitations, validation, and teardown
- [x] #3 Kubernetes manifests pass client-side rendering and schema validation where available
- [x] #4 Minikube manifests are grouped by infrastructure and application component without changing rendered resources
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Split the existing YAML documents by component under infrastructure/ and applications/, update nested Kustomizations and helper paths, then compare rendered resources and validate the result.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Caddy and media use hostPort only for HTTPS/HTTP and RTC respectively; private gRPC and operations remain on ClusterIP. coturn uses host networking for the local relay range. Local fixture credentials only. Validated 30 rendered resources with kubectl kustomize and successful kubectl apply --dry-run=server against the running Minikube API. just --list, bash -n, shellcheck, just docs, and git diff --check passed. Live image build, pod startup, HTTPS login, and WebRTC were not run; the temporary validation namespace was removed.

Reorganized Minikube YAML by component under infrastructure/ and applications/. Root Kustomize output is byte-for-byte identical to the previous 30-resource render. Updated helper paths and deployment guide. bash -n, shellcheck, just docs, and git diff --check passed. No live deployment was run for this file-only reorganization.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the local Minikube stack and Justfile lifecycle recipes, then grouped manifests by component under infrastructure/ and applications/. The reorganized Kustomize output is identical to the previous render; scripts and docs checks pass. Runtime behavior remains unverified.
<!-- SECTION:FINAL_SUMMARY:END -->
