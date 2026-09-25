# Feature modules

Product UI is grouped by the behavior it owns. `rooms` handles room discovery and management,
`sessions` handles the live session and lifecycle, `chat` handles persistent room conversations,
`files` handles scoped transfers, and `invites` handles code-based entry. Shared REST, WebSocket,
RTC, and collaboration transports live under `lib/`.

The workspace sidebar and mobile navigation browse rooms and sessions in dialogs. Creation opens
forms in dialogs from the plus control. Detail pages have direct URLs at `/room/[roomId]` and
`/session/[sessionId]`; there are no room or session index or `/new` pages. `/workspace` sends an
authenticated member to an available room or offers creation and code-based entry when no rooms
are available.

## Recording integration for M9

The session page resolves the caller's server capabilities before rendering controls. M9 should
add the recording capability and recording state to that response, then pass them into the session
client and stage. Recording controls must use the server's state, show a persistent text indicator
while recording, and keep the indicator visible to every participant. The UI needs explicit
starting, recording, stopping, and failed states; a locally toggled button cannot imply that a
recording exists. Recording artifacts should use the existing direct file transfer and authorized
download boundary, scoped to a recording context. Keep stable `data-testid` anchors on the
control, indicator, and artifact list for cross-service E2E tests.
