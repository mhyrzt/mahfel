# Browser tests

This directory is the home for Playwright browser tests. Add test cases under `tests/` as the cross-service flows become ready to verify.

From `frontend/`, install the pinned dependencies and Chromium once:

```bash
bun install --frozen-lockfile
bunx playwright install chromium
```

Start the local Compose stack, then run `bun run test:e2e`. The default origin is `https://mahfel.localhost`. Set `PLAYWRIGHT_BASE_URL` to use another origin. The configuration does not start or replace the application stack.

Failed-run traces and screenshots are written under `playwright/artifacts/`.
