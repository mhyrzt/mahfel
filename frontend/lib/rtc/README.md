# RTC client boundary

WebRTC signaling uses `mahfel-api`; negotiated media flows directly between the browser and
`mahfel-media`, with coturn as fallback. The live session client publishes microphone, camera,
and screen tracks with `mediasoup-client`. A separate receive transport reconciles the room's
producer snapshot, consumes remote tracks paused first, and resumes each consumer after the
browser attaches it. The snapshot is refreshed every three seconds while joined so reconnects
and producers published through another API replica are discovered.
