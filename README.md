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

## Feature comparison

How Mahfel compares with common meeting and virtual-classroom products. The Mahfel column reflects
implemented behavior, or a planned milestone where marked 🕒. Competitor entries are based on public
product documentation at the time of writing and can change by plan, edition, or version — ❌ means
"not advertised as a built-in feature" in the sources reviewed, not a definitive absence.

Legend: ✅ supported · ❌ not supported / not advertised · 🕒 on the Mahfel roadmap (not yet implemented)

| Feature | Mahfel | Google Meet | Jitsi Meet | BigBlueButton | Alocom | Skyroom |
| --- | :---: | :---: | :---: | :---: | :---: | :---: |
| Video meetings | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Large webinars / broadcasts | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ |
| Online classrooms | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Screen sharing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| In-meeting chat | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Collaborative whiteboard | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Collaborative notes | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ |
| File sharing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Presentation (PDF / images) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Video playback in presentation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Session recording | 🕒 | ✅ | ✅ | ✅ | ✅ | ✅ |
| Transcription / captions | 🕒 | ✅ | ❌ | ❌ | ✅ | ❌ |
| Breakout rooms / groups | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Polls / quizzes | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Raise hand | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Waiting room / lobby admission | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Attendance & participation report | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ |
| Invite links / join by code | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Host moderation (mute / remove / block) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| SSO (OIDC / SAML) | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Active speaker view | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Virtual backgrounds / blur | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Noise suppression | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| End-to-end encryption | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Phone dial-in (PSTN / SIP) | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Live streaming (RTMP) | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Native mobile apps | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Self-hosting / on-premises | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ |
| REST API & integrations | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Farsi-first / RTL interface | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| AI participant / assistant | 🕒 | ✅ | ❌ | ❌ | ❌ | ❌ |
| AI voice interaction | 🕒 | ❌ | ❌ | ❌ | ❌ | ❌ |

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
just minikube init
just minikube bootstrap
```

Initialize the cluster once. `just minikube bootstrap` builds and deploys the apps, then forwards
HTTPS until you stop the command. Host ports 80 and 443 must be free. Certificate trust and
forwarding may prompt for `sudo`. Open `https://mahfel.localhost` once forwarding starts. Run
`just minikube cert` if the Minikube CA changes.

See [getting started](docs/development/getting-started.md) for details. Run `just --list` for all
development commands.
