# Mahfel

Mahfel is a self-hosted platform for live classrooms and collaboration, built with
Rust, Next.js, and WebRTC. It brings video, shared whiteboards, collaborative
notes, file sharing, and recordings into a Farsi-first, RTL interface.

[Getting started](docs/development/getting-started.md) ·
[Documentation](docs/index.md) ·
[Report a bug](https://github.com/mhyrzt/mahfel/issues) ·
[MIT license](LICENSE)

## What you can do

- Teach live with audio, video, screen sharing, and classroom moderation.
- Work together on whiteboards and notes, and present PDFs, images, and video.
- Share files, invite participants, and review recordings and attendance reports.
- Host the application and artifact storage on your own infrastructure.
- Integrate with existing systems through REST APIs and Keycloak-based SSO.

Mahfel is under active development. See the [roadmap](docs/development/roadmap.md)
for planned work; roadmap entries are not promises of available functionality.

## Get started

Install Docker Engine with Compose v2, Just, and the Bun version pinned in
`frontend/package.json`. The [setup guide](docs/development/getting-started.md)
covers the full toolchain, including Rust for host-side backend development.

```bash
git clone https://github.com/mhyrzt/mahfel.git
cd mahfel
just bootstrap
just frontend install
just compose-up
```

Open `https://mahfel.localhost`. Bootstrap creates local environment files, starts
Caddy, and installs its local certificate authority; trusting the certificate may
prompt for `sudo`. The Compose stack builds the application images and starts the
supporting services. No private deployment repository is required.

The committed credentials and environment examples are for local development.
Configure your own secrets and networking before deploying to a shared host. See
[local deployment](docs/deployment/local.md) for configuration details.

## How it fits together

| Component | Role | Stack |
| --- | --- | --- |
| `mahfel-web` | Browser application | Next.js, React, TypeScript |
| `mahfel-api` | Application APIs and signaling | Rust |
| `mahfel-media` | Realtime media | Rust, mediasoup |
| `mahfel-worker` | Background jobs and recordings | Rust |

PostgreSQL stores durable application state, Redis coordinates ephemeral state,
RabbitMQ transports jobs, and S3-compatible storage holds artifact bytes.
See the [architecture documentation](docs/index.md) and [ADRs](docs/adr/).

## Contributing

Bug reports, documentation improvements, translations, and focused code changes
are welcome. [Open an issue](https://github.com/mhyrzt/mahfel/issues) to describe a
bug or discuss a substantial change before implementation. Include reproduction
steps and expected behavior for bugs; keep credentials and private classroom data
out of reports.

Fork the repository, create a branch, and submit a pull request with a short
explanation and validation results. Read [AGENTS.md](AGENTS.md) and the applicable
component instructions for architecture and coding conventions.

Useful local checks:

```bash
just frontend fmt
just frontend lint
just frontend typecheck
just frontend test
just backend fmt
just backend lint
just backend test
just docs
```

Run the checks relevant to your change. `just --list` lists root commands; use
`just frontend` or `just backend` to list component commands. Hosted CI is
currently disabled while the repository transitions to GitHub.

## License

Mahfel is licensed under the [MIT License](LICENSE).
