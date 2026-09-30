# ADR 0028: Generate MP4 exports from original recording bundles

## Status

Accepted

## Context

ADR 0027 preserves original tracks and collaboration data. The separately approved export scope
adds an automatically generated, downloadable video without changing live capture or its success
criteria. Interactive replay remains separately tracked under M9-06.

## Decision

Finalization accepts its original manifest and transactionally enqueues `recordings.export.v1`.
PostgreSQL owns export state independently of recording readiness. RabbitMQ has a dedicated export
queue and the existing worker runs one export concurrently by default, separately from ordinary
jobs. Retry generations, leases and source manifest identity fence artifact acceptance.

The standalone browser renderer reconstructs Yjs state, presentation/viewport history and chat.
Its typed replay state is separate from the versioned scene policy and rendering. The worker image
packages Bun, Playwright, Chromium, fonts, PDF.js and the renderer bundle; it does not depend on a
running Next.js instance or introduce a deployable service. Browser requests resolve only packaged
resources and archived assets through an intercepted virtual origin. Browser sandboxing stays enabled.

GStreamer decodes original fragments, composes RGBA media and transparent workspace frames, mixes
recorded microphones and displayed presentation audio, and encodes H.264/AAC fast-start MP4 at
1080p/30 fps. All timestamps derive from recorded integer clocks rather than rendering speed.
Pause duration is preserved with a slate and silence. Resume restores a fresh workspace snapshot.

Screen sharing takes priority, otherwise shared activity chooses the workspace surface. Camera
pages contain six tiles beside content or nine without content, rotating every ten seconds. Chat
appears as up to three six-second toasts, honoring edits and deletion while visible.

S3 stores private output bytes. Download authorization requires current room membership or an
administrator, with blocks enforced for members. Public visibility alone does not grant access.
Presigned download links expire after five minutes. Originals and accepted MP4s remain until
explicit deletion; full retention/deletion management belongs to M9-06.

## Consequences

Exports consume CPU, scratch disk and browser resources independently of the media plane. Limits
are configurable and fail explicitly. Temporary workspaces and incomplete multipart uploads are
cleaned up; duplicate delivery cannot replace an accepted export. A failed export leaves the
original replay bundle ready. Chromium requires a container runtime that supports its sandbox.

See [export development and operations](../development/recording-export.md).
