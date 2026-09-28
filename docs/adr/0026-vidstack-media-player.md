# ADR 0026: Use Vidstack for media playback

## Status

Accepted

## Context

Live sessions need synchronized video presentation, and the M9 recording pipeline will need
playback of stored recordings. The current stack has no media player UI. The browser already
renders PDF and image presentations and plays live WebRTC media through mediasoup, but progressive
MP4/WebM playback over a signed storage URL is a separate concern.

A player library is a production dependency, and ADR 0014 asks the frontend to avoid arbitrary
additional component libraries.

## Decision

Use `@vidstack/react` (Vidstack 1.x) for progressive media playback in the presentation surface and,
where suitable, for recording playback. Vidstack is a headless, accessible player built on web
components with a React adapter and supports React 18/19.

- Presentation and playback synchronization stay in our own `PresentationPlayer` component; the
  player library is only the rendering and control surface.
- Media bytes flow directly from S3-compatible storage to the browser through the signed download
  URL, preserving the existing architecture; they do not pass through Next.js or the API.
- This is a media player, not a second component system: layout, copy, and permissions remain in
  our components and design primitives.

## Consequences

- The frontend gains consistent, accessible media controls and a path to HLS/DASH and captions for
  future recording playback without re-implementing controls.
- The player adds client bundle weight; it is imported only on the presentation surface with a
  client-only dynamic import.
- Synchronized watch-together remains our responsibility and is not provided by the library.
- The dependency needs the same upkeep as other frontend dependencies; if recordings adopt a
  different delivery format, the player stays behind `PresentationPlayer`.

## Alternatives considered

- **Native `<video>` controls.** Sufficient for play/pause/seek of progressive files, but leaves
  captions, keyboard affordances, and future adaptive streaming to custom work. Kept as the fallback
  shape but not chosen, because recordings are expected to need a richer player.
- **Video.js or Plyr.** Comparable players, but Vidstack has a first-class React adapter and a
  headless element model that fits React 19 without wrapping a plugin architecture.
