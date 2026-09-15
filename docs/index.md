# Mahfel engineering documentation

Mahfel is a standalone, education-oriented realtime communication and collaboration platform. It
occupies the same broad category as BigBlueButton and Jitsi, extended toward persistent chat,
collaborative notes and whiteboards, files, recordings, transcripts, STT/TTS, and AI participants.

Mahfel is **not an LMS**. LMS products may integrate with it, but classrooms, sessions, media, and
collaboration remain useful without an LMS deployment.

The initial operational model deliberately contains four application components:
`mahfel-web`, `mahfel-api`, `mahfel-media`, and `mahfel-worker`. Capability modules stay within those
boundaries until production evidence justifies another deployable service.

Start with the [architecture overview](architecture/overview.md), then use the
[local deployment guide](deployment/local.md) and [engineering roadmap](backlog/roadmap.md).
