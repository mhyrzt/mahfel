---
id: TASK-56
title: M9-01 Recording architecture spike
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:32'
updated_date: '2026-09-28 13:12'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 64000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: SFU recording format/topology determines reliability and cost.

Implementation scope: Evaluate mediasoup plain transports/taps, per-track versus composite, RTP capture, ffmpeg/GStreamer, node-local buffering and failure model.

Dependencies: M3, M6, M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ADR selects strategy with measured prototype, resource profile, restart limitations and alternatives.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Prototype original encoded Opus/VP8/H264 capture through mediasoup plain transports and GStreamer Rust bindings. Measure synchronization and resource use at 1, 8 and 24 tracks, restart loss and bounded buffering. Record ADR and evidence before lifecycle implementation. No FFmpeg, transcoding, composition or generated video export.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Evaluate collaboration replay alongside A/V capture: timestamped whiteboard and text edits, snapshots, shared viewport changes, and references to stored presentation assets plus presenter slide changes. Interactive reconstruction is the baseline; a downloadable composited video requires a separate render/encode path and is optional.

Design clarification: The recording renderer may use a dedicated composition and layout independent of the live frontend. Evaluate this as a separate renderer over canonical media and collaboration timelines, not as a screenshot or clone of the live UI.

Replay-player clarification: The primary recording experience is a player that plays the captured audio/video tracks alongside an event-driven reconstruction of the shared room state. A single flattened video is not required; video export is a separate optional output.

Implemented native original-codec GStreamer/mediasoup prototype and ADR 0027. Real Opus/VP8/H264 fragments decode. Ten-minute fixture: 421 fragments; 2.542 ms initial cross-track skew; 5.090 ms max duration drift. Disk-backed 1/8/24 profiles pass with frozen sources and RTP drain; initial competing-build/live-endpoint 24-track run failed at 134 ms and is documented. Process kill retains finalized fragments; unsealed window 2.25 s. Pause/resume preserves 2.25 s while publisher runs. RTC tests, clippy and strict docs pass. Resource measurements and reproduction are in docs/development/recording-spike.md; image definitions updated, images not built.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Selected original per-track WebM/Matroska capture through loopback plain transports and GStreamer. Verified native codec readability, controlled ten-minute timing, track-count resource profiles, restart loss and pause gaps; documented limits and alternatives. No FFmpeg or generated video export.
<!-- SECTION:FINAL_SUMMARY:END -->
