---
id: TASK-168
title: Add browser-free authenticated recording image rendering
status: Done
assignee:
  - '@codex'
created_date: '2026-09-30 17:53'
updated_date: '2026-10-01 06:36'
labels: []
dependencies: []
ordinal: 176000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace production Chromium/Playwright rendering with native image rendering on the existing web port. Add API-key authenticated POST /api/internal/recording-render with versioned typed DTOs and bounded asset access, then integrate the worker export pipeline and remove bundled browser dependencies. Preserve collaboration, A/V timing and export claim fencing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The endpoint validates API keys and typed bounded input and returns PNG/JPEG/self-contained SVG without Chromium or Playwright
- [x] #2 Whiteboards, rich notes including Persian, archived image/PDF pages and slates render with native libraries
- [x] #3 Worker exports consume the HTTP renderer and production worker packaging contains no frontend bundle, Bun, Chromium or Playwright
- [x] #4 Relevant endpoint, replay/export, packaging and documentation checks pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Record a superseding ADR and typed image-render contract. 2. Implement native SVG/raster drawing, bounded archived-asset reads and authentication on the existing web listener. 3. Replace browser subprocess rendering in the worker while preserving deterministic replay and scene composition. 4. Remove obsolete production browser packaging/configuration and verify real rendered images and MP4 output.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-10-01: Native NextRequest/NextResponse endpoint split into authentication, bounded request parsing, capacity, validation, asset loading, raster/vector rendering and HTTP responses. Production Bun 1.4.2 web image returns images without Chromium. Container visual verification exposed WOFF2 missing glyphs; fixed with a private upstream SIL-licensed Vazirmatn TTF traced into the standalone web image. Empty overlays/blank paragraphs have regression coverage. Rich Persian notes preserve formatting, direction and ordered-list start values. Native Rust replay tests cover seeking, generations, presentation clock, chat changes, gaps and camera/screen policy. Full real MP4 integration passed against the production Bun/Next.js container with PDF/image assets and both streams decoded to EOS. Web/worker production builds pass; worker image executable/artifact checks confirm no Bun, Node, Chromium or frontend renderer. Frontend suite 167 tests plus the new empty-text regression (13 server cases); typecheck/lint/format, backend workspace unit tests and strict Clippy, SQL lint, protobuf, docs, Compose and Minikube rendering pass. No production deployment performed; v1.0.0 release scope is Git tags and GitLab release entries only.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced worker Chromium/Playwright/frontend packaging with authenticated native image rendering on the existing web port. Preserved Rust/GStreamer recorded-clock replay, media composition and fenced export lifecycle; verified real MP4 output and private Persian font packaging. ADR 0030 documents web availability dependency and native visual differences.
<!-- SECTION:FINAL_SUMMARY:END -->
