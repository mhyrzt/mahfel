# Recording MP4 exports

The worker exports newly finalized usable recordings. Recording readiness and export status are
independent. Current room members can download accepted MP4s; managers can retry failed exports.
Public visibility alone is insufficient. Originals survive rendering failures.

## Local development

Install GStreamer base/good/bad/ugly and libav plugins. `x264enc`, `avenc_aac`, `pngdec`,
`compositor` and original media decoders must be available. The web renderer requires Poppler's
`pdftocairo` and fontconfig. Production Dockerfiles install the native packages.

```sh
cd frontend
bun install --frozen-lockfile
MAHFEL_RENDER_API_KEY=local-render-key MAHFEL_RENDER_ASSET_ORIGINS=http://localhost:9000 bun run dev
cd ../backend
MAHFEL_WORKER__EXPORTS__RENDERER_API_KEY=local-render-key just dev-worker
```

Use a dedicated secret in production. Asset origins must match the storage adapter's internal
endpoint. Compose uses `http://rustfs:9000` and a development-only key. No Chromium installation or
browser bundle is required. Browser E2E dependencies live under repository-level `e2e/`.

`recording-assets:prepare` copies the browser PDF.js worker and Excalidraw fonts into
`public/recording-renderer/`, alongside the browser Vazirmatn WOFF2 font. Development, build and
Storybook commands prepare these automatically. Native Sharp/Pango rendering instead reads a
private TTF font from `features/recordings/server-render/assets/`, packaged by Next.js output
tracing. The worker contains neither set of frontend assets.

## Configuration

| Component | Setting | Purpose |
| --- | --- | --- |
| Web | `MAHFEL_RENDER_API_KEY` | Required bearer API key; absent configuration denies rendering |
| Web | `MAHFEL_RENDER_ASSET_ORIGINS` | Comma-separated exact HTTP(S) storage origins; no default allowlist |
| Worker | `MAHFEL_WORKER__EXPORTS__RENDERER_URL` | Default `http://localhost:3000/api/internal/recording-render` |
| Worker | `MAHFEL_WORKER__EXPORTS__RENDERER_API_KEY` | Must match the web rendering key |

Worker settings use the existing layered `worker.exports` configuration. The Docker profile points
to `http://mahfel-web:3000/api/internal/recording-render`. No separate listener is introduced.
Requests have a 35-second worker timeout and 30-second web abort signal. Two renders may run
concurrently per web process. HTTP 5xx/429 and transport failures use the export retry lifecycle;
invalid rendering input fails explicitly.

Worker defaults are one concurrent export, 20 GiB scratch, 8 GiB output, 64 active microphones and a
24-hour processing timeout. Scratch reserves twice the output limit before staging source bytes.
Increase limits only with matching capacity. Claim loss and drain expiry terminate export processes.
Temporary workspaces and failed multipart uploads are cleaned up. PostgreSQL owns export state; S3
owns accepted artifacts. Five-minute internal presigned asset URLs are refreshed every four minutes.

## Image endpoint contract

`POST /api/internal/recording-render` requires `Authorization: Bearer <render-key>` and
`Content-Type: application/json`. It does not use browser cookies or session authentication. The
web container executes Next.js with Bun; `runtime = "nodejs"` selects Next.js's native API mode.

```json
{
  "version": 1,
  "output": {"format": "png", "width": 1280, "height": 720, "background": "#ffffff"},
  "content": {
    "type": "notes",
    "direction": "rtl",
    "document": {"blocks": [{"type": "paragraph", "runs": [{"text": "یادداشت جلسه"}]}]},
    "viewport": {"scroll_y": 0}
  }
}
```

The authoritative DTO/parser are `frontend/features/recordings/server-render/types.ts` and
`validation.ts`. Content types are `whiteboard`, `notes`, `document_page`, `image` and `slate`.
Whiteboards contain typed drawing elements and a viewport. Notes contain structured blocks,
formatted runs and paragraph direction. PDF pages are one-based. Archived assets use
`{ "url": "https://storage.example/archive?...", "sha256": "<64 lowercase hex digits>" }`.
Optional annotations add text panels after content clipping. `output.content_height` reserves
space for camera tiles. JPEG requires an opaque background and accepts optional `quality`.

Responses are binary `image/png`, `image/jpeg` or self-contained `image/svg+xml`, with
`Cache-Control: no-store` and `X-Renderer-Version: native-v1`. Errors are JSON `{code, message}`:
401 for authentication, 413 for limits, 415 for content type, 422 for invalid data, and 503 for
unavailable/unconfigured/busy rendering. Busy responses include `Retry-After: 1`.

Limits include 8 MiB requests, 1920×1080 output, 2000 drawing elements, 20000 total points, bounded
drawing area, 1000 notes blocks, 100000 text characters, 32 assets, 50 MiB per asset, 64 MiB total
asset bytes and 16 MiB output. Redirects and origins outside the allowlist are rejected. SHA-256 is
checked before decoding. Assets are PNG/JPEG or PDF as appropriate. Arbitrary HTML, uploaded SVG
and browser execution are unsupported.

Sharp/Pango, RoughJS, perfect-freehand and Poppler render without a browser. Persian text is shaped
natively. SVG embeds raster text and assets. Native typography and drawings can differ from browser
Excalidraw/Lexical visuals; [ADR 0030](../adr/0030-native-recording-image-rendering.md) records this.

## Export contracts and replay

- `GET /v1/sessions/{session_id}/recordings/{recording_id}/export`: state/progress.
- `POST` on that path: idempotent manager retry/enqueue.
- `GET .../export/download`: authorized five-minute presigned MP4 URL.
- `recordings.export.v1`: durable outbox delivery with independent worker intake and DLQ.

Rust/Yrs reconstructs documents, presentation history and chat using recorded integer clocks.
GStreamer composes camera/screen/presentation media with PNG overlays, mixes audio and encodes
H.264/AAC fast-start MP4. Unchanged drawing requests reuse the previous PNG. Screen sharing takes
priority; cameras rotate every ten seconds, six alongside content or nine without it. Pause and
outage slates preserve elapsed time. Resume restores a fresh snapshot. Claim fencing and acceptance
remain independent of rendering.

The internal `mahfel-worker render-export <workspace>` command consumes prepared source files and
the configured HTTP renderer. It opens no listener. Interactive playback and deletion are documented
in [the recordings reference](../deployment/recordings.md).

## Native fixture verification

Start the web renderer with a test key and allow the fixture asset origin:

```sh
cd frontend
MAHFEL_RENDER_API_KEY=fixture-render-key MAHFEL_RENDER_ASSET_ORIGINS=http://127.0.0.1:3011 bun run dev
```

Then run the integration fixture:

```sh
cd backend
MAHFEL_RECORDING_EXPORT_TEST=1 \
MAHFEL_WORKER__EXPORTS__RENDERER_URL=http://127.0.0.1:3000/api/internal/recording-render \
MAHFEL_WORKER__EXPORTS__RENDERER_API_KEY=fixture-render-key \
cargo test -p mahfel-worker --test recording_exports
```

The test owns a loopback-only asset server on port 3011. It generates camera and microphone
fragments, renders whiteboard, image/PDF/video presentations, chat, pause and resumed Persian notes,
then verifies the MP4 by decoding both streams to EOS. It requires an actual running web renderer
and native plugins. Every production export checks duration and decoding before uploading.
