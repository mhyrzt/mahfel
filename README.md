# Mahfel

Mahfel is a standalone, education-oriented realtime communication and collaboration
platform. It is designed for classrooms, conferencing, persistent chat,
collaborative notes and whiteboards, files, recordings, transcripts, and AI
participants. It can integrate with an LMS, but it is not itself an LMS.

The repository currently contains the engineering foundation only:

- `mahfel-web` — Next.js browser application
- `mahfel-api` — Rust control plane
- `mahfel-media` — Rust mediasoup media plane
- `mahfel-worker` — Rust background worker

See the [documentation](docs/index.md) for architecture, networking, deployment,
and the implementation roadmap.

## Quick start

```bash
cp .env.example .env
just frontend install
just compose-up
```

Run `just --list` for all development commands.
