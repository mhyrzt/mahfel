# Browser tests

This directory is the home for Playwright browser tests. Add test cases under `tests/` as the cross-service flows become ready to verify.

From `e2e/`, install the pinned dependencies and both configured browsers once:

```bash
bun install --frozen-lockfile
bunx playwright install chromium firefox
```

Start the local Compose stack, then run `bun run test` (or `just e2e` from the repository root). The default origin is `https://mahfel.localhost`. Set `PLAYWRIGHT_BASE_URL` to use another origin. The configuration does not start or replace the application stack. Set `MAHFEL_E2E_PASSWORD` to the local development account password and optionally set `MAHFEL_E2E_USERNAME` (default `misato`).

The recording regression creates a room and recorded session through Chromium with deterministic microphone, camera and screen devices, then edits collaborative notes. Playback is checked in Chromium and Firefox, including audio/video progress, pause, microphone mute and seeking from the screen share into notes. The local outsider account `shinji` must receive 404 for playback, media-window and export-download URLs. Chromium also downloads the completed MP4 and checks its byte count and container header. Successful screenshots and the MP4 are attached under `e2e/artifacts/`. This flow verifies Firefox replay; Firefox live publishing is a separate media transport check.

Failed-run traces and screenshots are written under `e2e/artifacts/`.

Scenario specs can be run independently, for example:

```bash
bunx playwright test tests/persistent-chat.spec.ts --project=chromium
```

| Spec | Scenario |
| --- | --- |
| `session-media-recovery.spec.ts` | Cross-replica camera/audio, screen sharing, recovery, peer rejoin and device cleanup |
| `device-permissions.spec.ts` | Permission denial and grant/retry in Chromium |
| `session-admission-moderation.spec.ts` | Lobby approval, enforced mute/unmute and removal across API replicas |
| `persistent-chat.spec.ts` | Durable messages, attachments, edits and deletion without repeated history reloads |
| `invite-links.spec.ts` | Confirmation, approval, rotation and revocation |
| `session-lifecycle.spec.ts` | Creation, editing, lifecycle transitions and participation summaries |
| `session-lifecycle-rtl.spec.ts` | Persian scheduling, keyboard interaction and narrow-screen layout |

These scenarios share login and API setup helpers under `helpers/`. Cross-replica scenarios
require the second client's `X-Acceptance-Replica: 2` header to route to another API instance.
Set `MAHFEL_E2E_AUDIO_FIXTURE` to a continuous WAV fixture to enable the overlapping-speaker
assertions in Chromium. Media capture is synthetic; signaling, persistence and media transport
use the running services. Controlled transport-failure injection does not prove packet-loss or
TURN-only recovery.
