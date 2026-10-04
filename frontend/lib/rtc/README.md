# RTC client boundary

WebRTC signaling uses `mahfel-api`; negotiated media flows directly between the browser and
`mahfel-media`, with coturn as fallback. The live session client publishes microphone, camera,
and screen tracks with `mediasoup-client`. A separate receive transport reconciles the room's
producer snapshot, consumes remote tracks paused first, and resumes each consumer after the
browser attaches it. The snapshot is refreshed every three seconds while joined so reconnects
and producers published through another API replica are discovered.

The session header samples the existing send/receive transports' browser `getStats()` reports
every three seconds without sending extra network probes. Media latency is the selected ICE
candidate pair's round-trip time, not an ICMP ping or a speed test. Jitter and packet loss use
active incoming RTP streams and recent remote receiver reports for outgoing media. Incoming
packet loss uses differences between consecutive counters; outgoing loss uses `fractionLost`
from receiver reports received within the last 15 seconds. Missing statistics remain unavailable.

The displayed quality is a local estimate using the worst available stream/transport measurement.
It is poor at RTT >= 700ms, jitter >= 80ms or loss >= 5%; fair at RTT >= 300ms, jitter >= 30ms
or loss >= 2%; otherwise good when at least one measurement exists. These are application
heuristics, not a guarantee of media quality. Offline/signaling failures and interrupted media
take precedence over the estimate. Sampling stops when the session is inactive, signaling is
disconnected, the browser is offline, or the component unmounts; measurements are not logged.
