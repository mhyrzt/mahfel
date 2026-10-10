# Mahfel

Mahfel brings live classrooms, collaborative whiteboards and notes, file sharing,
and recordings into one self-hosted platform with a Farsi-first, RTL interface.
Run it on your own infrastructure and connect it to your LMS through REST APIs
and SSO.

## Why Mahfel

- **Built for teaching:** combine video, screen sharing, PDF/image presentations,
  shared notes, and whiteboards in the same classroom.
- **Designed for Farsi-speaking classrooms:** a Farsi-first, RTL interface.
- **Control your deployment:** self-host the platform and its artifact storage
  on infrastructure you operate.
- **Connect your existing systems:** REST integrations and SSO let Mahfel fit
  alongside your LMS without replacing it.
- **Keep learning after the session:** recordings, file sharing, and attendance
  reports support review and follow-up.

Transcription, AI participants, and AI voice interaction are planned; the
comparison below distinguishes supported features from future work.

## Application components

Mahfel runs as four application components:

- `mahfel-web` — Next.js browser application
- `mahfel-api` — Rust control plane
- `mahfel-media` — Rust mediasoup media plane
- `mahfel-worker` — Rust background worker

See the [documentation](docs/index.md) for architecture, networking, deployment,
and the implementation roadmap.

## Feature comparison

How Mahfel compares with common meeting and virtual-classroom products. The Mahfel column reflects
implemented behavior, or planned features where marked 🕒. Competitor entries are based on public
product documentation at the time of writing and can change by plan, edition, or version — ❌ means
"not advertised as a built-in feature" in the sources reviewed, not a definitive absence.

Legend: ✅ supported · ❌ not supported / not advertised · 🕒 planned for Mahfel (not yet implemented)

| Feature                                 | Mahfel | Google Meet | Jitsi Meet | BigBlueButton | Alocom | Skyroom |
| --------------------------------------- | :----: | :---------: | :--------: | :-----------: | :----: | :-----: |
| Video meetings                          |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Large webinars / broadcasts             |   🕒   |     ✅      |     ✅     |      ❌       |   ✅   |   ✅    |
| Online classrooms                       |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Screen sharing                          |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| In-meeting chat                         |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Collaborative whiteboard                |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Collaborative notes                     |   ✅   |     ❌      |     ✅     |      ✅       |   ❌   |   ❌    |
| File sharing                            |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Presentation (PDF / images)             |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Video playback in presentation          |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Session recording                       |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Transcription / captions                |   🕒   |     ✅      |     ❌     |      ❌       |   ✅   |   ❌    |
| Breakout rooms / groups                 |   🕒   |     ✅      |     ✅     |      ✅       |   ✅   |   ❌    |
| Polls / quizzes                         |   🕒   |     ✅      |     ✅     |      ✅       |   ✅   |   ❌    |
| Raise hand                              |   🕒   |     ✅      |     ✅     |      ✅       |   ❌   |   ❌    |
| Waiting room / lobby admission          |   ✅   |     ✅      |     ✅     |      ✅       |   ❌   |   ❌    |
| Attendance & participation report       |   ✅   |     ✅      |     ❌     |      ✅       |   ✅   |   ❌    |
| Invite links / join by code             |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Host moderation (mute / remove / block) |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| SSO (OIDC / SAML)                       |   ✅   |     ✅      |     ✅     |      ✅       |   ❌   |   ❌    |
| Active speaker view                     |   ✅   |     ✅      |     ✅     |      ✅       |   ❌   |   ❌    |
| Virtual backgrounds / blur              |   🕒   |     ✅      |     ✅     |      ✅       |   ❌   |   ❌    |
| Noise suppression                       |   🕒   |     ✅      |     ✅     |      ❌       |   ❌   |   ❌    |
| End-to-end encryption                   |   🕒   |     ✅      |     ✅     |      ❌       |   ❌   |   ❌    |
| Phone dial-in (PSTN / SIP)              |   ❌   |     ✅      |     ❌     |      ❌       |   ❌   |   ❌    |
| Live streaming (RTMP)                   |   🕒   |     ✅      |     ✅     |      ❌       |   ❌   |   ✅    |
| Native mobile apps                      |   🕒   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Self-hosting / on-premises              |   ✅   |     ❌      |     ✅     |      ✅       |   ✅   |   ✅    |
| REST API & integrations                 |   ✅   |     ✅      |     ✅     |      ✅       |   ✅   |   ✅    |
| Farsi-first / RTL interface             |   ✅   |     ❌      |     ❌     |      ❌       |   ✅   |   ✅    |
| AI participant / assistant              |   🕒   |     ✅      |     ❌     |      ❌       |   ❌   |   ❌    |
| AI voice interaction                    |   🕒   |     ❌      |     ❌     |      ❌       |   ❌   |   ❌    |

Recording includes original tracks, authorized interactive replay and automatic workspace MP4 exports.
Managers can delete a recording and its artifacts; see
[recording operations](docs/deployment/recordings.md) and
[MP4 export operations](docs/development/recording-export.md).

## Quick start

```bash
just bootstrap
just frontend install
just compose-up
```

Open `https://mahfel.localhost`. `just bootstrap` installs the Compose Caddy CA in your host
trust store and may prompt for `sudo`.

See [getting started](docs/development/getting-started.md) for details. Run `just --list` for all
development commands.

## Deployment repository

`devops/` is a Git submodule pointing to `hamdars/devops`, which contains Rastploy and shared
deployment configuration. Initialize the pinned revision after cloning Mahfel:

```bash
git submodule update --init devops
```

Access to the private `hamdars/devops` repository is required. Application builds do not require
the submodule, and its files are excluded from Mahfel's Docker build context.
