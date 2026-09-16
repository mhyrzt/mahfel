# Runtime data flows

## Classroom join

```mermaid
sequenceDiagram
  participant B as Browser
  participant A as mahfel-api
  participant P as PostgreSQL
  participant R as Redis
  participant M as mahfel-media
  B->>A: Authenticate and request classroom join
  A->>P: Authorize membership and role
  A->>R: Read or acquire room placement
  A->>M: gRPC ensure room on selected node
  M-->>A: Router and transport capabilities
  A-->>B: Session metadata and signaling channel
```

## WebRTC signaling

```mermaid
sequenceDiagram
  participant B as Browser mediasoup-client
  participant A as mahfel-api WebSocket
  participant M as mahfel-media gRPC
  B->>A: Request transport
  A->>M: CreateWebRtcTransport
  M-->>A: ICE, DTLS, SCTP parameters
  A-->>B: Transport parameters
  B->>A: Connect/produce/consume signals
  A->>M: Matching internal RPC
  M-->>A: Control result
  A-->>B: Signaling result
```

## Direct media path

```mermaid
flowchart LR
  A[Teacher browser] <-->|SRTP over UDP preferred| M[mahfel-media SFU]
  B[Student browser] <-->|SRTP over UDP preferred| M
  C[Student browser] <-->|SRTP over UDP preferred| M
  API[mahfel-api] -. signaling only .-> M
```

## TURN fallback

```mermaid
flowchart LR
  Browser[Restricted browser network] <-->|TURN allocation| Turn[coturn]
  Turn <-->|relayed media| Media[mahfel-media]
  API[mahfel-api] -. credentials and signaling .-> Browser
```

## Background job processing

```mermaid
sequenceDiagram
  participant A as mahfel-api
  participant P as PostgreSQL
  participant Q as RabbitMQ
  participant W as mahfel-worker
  participant S as S3
  A->>P: Commit domain change and outbox row
  A->>Q: Publish with confirmation
  Q->>W: Deliver versioned job
  W->>W: Check idempotency key
  W->>S: Process/read/write artifact
  W->>P: Persist result
  W->>Q: Manual acknowledgement
```

Transient failures are retried; poison jobs are routed to a dead-letter queue (DLQ).

## File transfer

The browser asks `mahfel-api` to authorize an operation and issue a short-lived presigned URL. The
browser transfers bytes directly to S3. The API persists metadata and permissions in PostgreSQL;
large file bodies are not proxied through API or Next.js under the normal path.
