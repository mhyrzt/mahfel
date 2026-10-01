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
