---
id: TASK-174
title: Build uv deployment management CLI with TOML config
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 18:06'
updated_date: '2026-10-02 18:22'
labels: []
dependencies: []
ordinal: 182000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the shell encryption helper with a typed Python package managed by uv. Add a reusable deployment CLI for secret encryption, manifest render/validation, deployment status and explicit Argo sync. Read CLI settings and private credential inputs from config.toml, preserve runtime application env contracts, update Python ignores and document usage. Keep changes uncommitted for review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 uv package and lockfile provide a runnable mahfel-deploy command and focused development checks
- [x] #2 CLI configuration and encryption credentials use typed TOML models with secret-safe validation and no dotenv loading
- [x] #3 Secret encryption preserves existing credential checks, generates six compatible Secrets and atomically publishes only valid SOPS ciphertext
- [x] #4 Render/validate/status/sync commands preserve Argo migration hooks and explicitly configured target contexts
- [x] #5 Old helper and obsolete input templates are removed; callers, docs and Python gitignore entries are updated
- [x] #6 Focused CLI, encryption, failure-path, typing/lint and docs checks pass without live deployment
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Create the uv package, typed TOML configuration and private credential models. 2. Move encryption into Python with in-memory plaintext and atomic ciphertext publication. 3. Add manifest and Argo operations with explicit context checks. 4. Replace obsolete templates and update commands, documentation and Python ignores. 5. Run focused crypto, failure, typing, lint, package and documentation checks; leave all changes uncommitted.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented deploy/cli as a uv package with Typer, Rich and Pydantic dataclasses. Private config.toml replaces three credential env templates. Six Secrets are built in memory; SOPS ciphertext is structurally validated and atomically published. Render uses the existing CMP shell helper with a TOML age identity. Argo sync verifies the same Application UID via both configured contexts and preserves full sync hooks. Validation: 25 focused tests passed, including real disposable SOPS crypto; Ruff check/format, strict mypy for seven source files, uv sdist/wheel build, just deploy help, strict MkDocs, docker compose config, gitignore checks and git diff --check passed. No live cluster/VPS operation, deployment, commit or push performed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Created the uv deployment CLI with typed config.toml settings, secret encryption, manifest validation and explicit-context Argo status/full sync. Removed obsolete encryption inputs, updated Python ignores and runbooks. All focused local checks passed; changes remain uncommitted for review.
<!-- SECTION:FINAL_SUMMARY:END -->
