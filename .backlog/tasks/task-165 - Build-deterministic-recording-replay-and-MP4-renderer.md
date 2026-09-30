---
id: TASK-165
title: Build deterministic recording replay and MP4 renderer
status: Done
assignee:
  - '@codex'
created_date: '2026-09-29 09:41'
updated_date: '2026-09-29 13:12'
labels: []
dependencies:
  - TASK-60
priority: medium
ordinal: 173000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the approved full-workspace 1080p MP4 export with adaptive scenes, cameras, audio, presentation playback and six-second stacked chat toasts.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Real replay fixtures produce playable synchronized MP4 output with pauses and gaps.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Replay immutable archived snapshots/events on the session clock; isolate offline Chromium workspace rendering in the worker; compose original camera/screen/video and mixed microphone audio with GStreamer into 1080p30 H264/AAC faststart MP4; bound inputs, scratch, output and processes; verify decoded output and document runtime packaging.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented replay/SceneV1, native media compositor, archived Lexical/Excalidraw/PDF/images/video, pause/resume, rotating camera layouts, six-second chat toasts and explicit partial coverage. Native 960p fixture passed with all workspace surfaces; final 1080p clock validation in progress.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented deterministic archived workspace replay and SceneV1 composition with GStreamer 1080p30 H264/AAC MP4 output, notes/whiteboard/PDF/images/video, microphones, cameras/screens, pause gaps and chat toasts. Real final 1080p fixture passed full decode-to-EOS and audio/video timing checks; renderer image, Storybook and strict source checks passed. Runtime container smoke verification remains tracked in TASK-166.
<!-- SECTION:FINAL_SUMMARY:END -->
