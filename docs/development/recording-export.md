# Recording MP4 exports

The worker automatically exports a newly finalized usable recording. Recording readiness and
export status are independent. Room members can download ready MP4s from the recording action in
the room session list; managers can retry failed exports. Public session access is insufficient.

## Local development

Install the native GStreamer base/good/bad/ugly and libav plugins. `x264enc`, `avenc_aac`, `pngdec`,
`compositor` and the original media decoders must be available.

```sh
cd frontend
bun install --frozen-lockfile
bunx playwright-core install chromium
bun run recording-renderer:build
cd ../backend
just dev-worker
```

Build the renderer before starting Storybook; its scripts also build it automatically, and the
Next.js `dev`/`build` scripts build it so the interactive player can serve the renderer font, PDF
worker and Excalidraw fonts from `public/recording-renderer/`. Worker images package the standalone
bundle and a matching Playwright Chromium installation. Browser sandboxing
is enabled. Compose uses the version-pinned official Playwright seccomp profile in
`deploy/worker/chromium-seccomp.json`. Minikube installs it under the kubelet seccomp directory
before applying the worker; other Kubernetes installations must provision
`/var/lib/kubelet/seccomp/mahfel/chromium.json` on worker nodes. The profile permits the namespace
syscalls Chromium requires while retaining its default deny policy. See the
[Playwright Docker sandbox guidance](https://playwright.dev/docs/docker#crawling-and-scraping).
The worker runs as UID 10001 with 256 MiB shared memory for Chromium.

## Configuration

`worker.exports` uses the existing layered TOML/environment settings. Defaults are one export
per worker, 20 GiB scratch per job, 8 GiB output, 64 simultaneously recorded microphones and a
24-hour processing timeout. Exhausting limits fails the export explicitly. Increase limits only
with corresponding disk/memory/CPU capacity. Scratch reserves twice the output limit for MP4
faststart finalization before staging source bytes. Uploaded originals remain available after failures.

The renderer directory is `../frontend/recording-renderer-dist` locally and
`/app/recording-renderer` in containers. Scratch is `/tmp/mahfel-exports`. Scratch is disposable;
PostgreSQL and S3 remain authoritative. Export processes are terminated on lost claims or drain
expiry. Every five minutes the worker removes scratch directories and unaccepted uploaded
artifacts older than one day, excluding active claims. Normal completion and cancellation remove
temporary workspaces immediately. Bucket initialization also installs a one-day abort policy for
incomplete multipart uploads under `recordings/`; configure this policy on existing external
buckets. Kubernetes gives the worker a bounded 20 GiB scratch volume.

## Contracts

- `GET /v1/sessions/{session_id}/recordings/{recording_id}/export`: export state and progress.
- `POST` on the same path: idempotent manager retry/enqueue for a ready original recording.
- `GET .../export/download`: authorized five-minute presigned MP4 URL.
- `recordings.export.v1`: durable outbox delivery with independent worker intake and DLQ.

Interactive playback, deletion and retention live in
[the recordings reference](../deployment/recordings.md). Replay playback reuses the same immutable
replay engine and archived workspace assets as the export renderer, but serves them to the browser as
short-lived presigned URLs instead of staging them in the worker.

The scene/replay interfaces live in `frontend/features/recordings/replay`. The browser renderer
lives beside them. The Rust adapter in `mahfel-worker` owns GStreamer and temporary artifact I/O.
Protocol operations initialize a recording, render a timestamp and dispose resources. Frame
responses contain `SceneV1`, presentation video timing and a transparent PNG overlay.

The internal `mahfel-worker render-export <workspace> <renderer-directory>` command is for native
fixture tests and worker subprocess execution. It accepts only prepared local data and makes no
public listener available. No FFmpeg command or browser recording capture is involved.

## Native fixture verification

Build the worker and renderer, then run the integration fixture with native plugins installed:

```sh
cd backend
MAHFEL_RECORDING_EXPORT_TEST=1 cargo test -p mahfel-worker --test recording_exports
```

The fixture generates VP8 camera and Opus microphone fragments, renders whiteboard, chat,
a pause and resumed Persian notes, and decodes both output streams to EOS. Every production
export also decodes to EOS and checks the output duration before uploading it.
