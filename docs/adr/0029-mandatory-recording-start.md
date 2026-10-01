# ADR 0029: Gate auto-recorded session start on capture readiness

## Status

Accepted. Supersedes the startup-failure decision in ADR 0027.

## Context

ADR 0027 lets capture startup fail after a session becomes live. For sessions with
`auto_record=true`, this can leave a live class without the mandatory recording.
The media node cannot capture participant tracks before participants publish, but
the control plane can verify the capture dependencies before admitting the class.

## Decision

A recorded start obtains a PostgreSQL lease while the session remains Scheduled.
The lease expires after 15 seconds and fences concurrent starts. The selected
media node checks capture codecs, capacity and writable spool, writes, reads and
deletes a private object in the recording bucket, then creates the media room. The
check has an eight-second deadline within the lease so it can return before the
browser's ten-second request timeout. Only a
healthy result under the matching, unexpired lease commits Live. The existing
session transition creates the recording intent in that commit. A failed check
releases the lease and leaves the session Scheduled for an explicit retry. An API
crash leaves only an expiring lease; a later request can reclaim it. A concurrent
preference edit or session transition makes the commit fail.

`auto_record=false` starts in one database transaction without preflight. After
the recorded session becomes Live, capture interruption leaves the class live,
shows a warning and records a coverage gap while recovery retries continue.

## Consequences

The readiness check verifies media and storage availability at start time. It
cannot guarantee that future device publication or network delivery will succeed;
post-start warnings and partial coverage remain essential. The preflight creates
only temporary spool and S3 probe objects, and an idle media room may exist after
a failed database commit until normal media cleanup.
