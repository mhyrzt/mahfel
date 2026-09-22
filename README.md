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
just bootstrap
just frontend install
just compose-up
```

Open `https://mahfel.localhost`. `just bootstrap` installs the Compose Caddy CA in your host
trust store and may prompt for `sudo`.

To use Minikube instead:

```bash
just minikube-init
just minikube-bootstrap
```

Initialize the cluster once. `just minikube-bootstrap` builds and deploys the apps, then forwards
HTTPS until you stop the command. Host ports 80 and 443 must be free. Certificate trust and
forwarding may prompt for `sudo`. Open `https://mahfel.localhost` once forwarding starts. Run
`just minikube-cert` if the Minikube CA changes.

See [getting started](docs/development/getting-started.md) for details. Run `just --list` for all
development commands.
