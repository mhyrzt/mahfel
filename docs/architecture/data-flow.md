# Runtime data flows

## Authentication and session

```mermaid
sequenceDiagram
  participant B as Browser
  participant W as mahfel-web (NextAuth + BFF)
  participant K as Keycloak
  participant A as mahfel-api
  participant P as PostgreSQL
  B->>W: Start login
  W->>K: Authorization code with PKCE and state
  K-->>W: Code, then tokens (server-side exchange)
  W->>A: POST /v1/auth/provision with the access token
  A->>K: Fetch JWKS and verify the token
  A->>P: Upsert user and identity by (issuer, subject)
  A->>K: Link mahfel_user_id onto the Keycloak identity
  A-->>W: Application profile
  W->>K: Refresh the access token
  K-->>W: Token carrying app_user_id
  W-->>B: httpOnly session cookie
  B->>W: GET /api/backend/v1/...
  W->>A: Forward with Authorization: Bearer
  A->>A: Build the principal from app_user_id (no identity lookup)
  A-->>W: Response
```

## Session lifecycle

```mermaid
sequenceDiagram
  participant C as Browser or service principal
  participant A as mahfel-api
  participant P as PostgreSQL
  C->>A: POST /v1/rooms/{room_id}/sessions
  A->>P: Require manager membership of the room
  A->>P: Insert session (scheduled)
  A-->>C: 201 session
  C->>A: POST /v1/sessions/{session_id}/start
  A->>P: Require manager membership, then update to live
  A-->>C: 200 session (live)
  C->>A: POST /v1/sessions/{session_id}/end
  A->>P: Require manager membership, then update to ended
  A-->>C: 200 session (ended)
```

A session moves `scheduled` to `live` to `ended`, or `scheduled` to `cancelled`; any other
transition is rejected with `409` and leaves the row unchanged. At most one session per room may be
`live`; a partial unique index makes a concurrent second start fail with `409` instead of creating a
second live session. Every accepted action writes a structured `audit` log line. Sessions are
scheduled and controlled by room managers only. Member-facing join lives in the session-join flow
below, and connected time is recorded durably in `mahfel.participation_intervals` (M4-03): a join
opens an interval, a leave or any transport disconnect closes it, and ending a session sweeps any
interval still open so none outlives its session.

## Invite links

```mermaid
sequenceDiagram
  participant M as Manager browser
  participant A as mahfel-api
  participant P as PostgreSQL
  M->>A: POST /v1/rooms/{room_id}/invite-links
  A->>P: Require manager membership of the room
  A->>P: Insert link (generated code, unique index decides)
  A-->>M: 201 link with the canonical code
  M->>A: POST /v1/invite-links/{link_id}/rotate
  A->>P: Revoke previous and insert replacement in one transaction
  A-->>M: 200 replacement link
```

Management is capability-gated for the target and audited. A code is a bearer capability, so its
expiry and revocation are enforced on redemption rather than stored as a derived flag, and the code
never appears in logs or metric labels (ADR 0021). Redemption and the confirmation flow that follows
it belong to M4-14 (TASK-130).

## Session join

```mermaid
sequenceDiagram
  participant B as Browser
  participant A as mahfel-api
  participant P as PostgreSQL
  participant R as Redis
  participant M as mahfel-media
  B->>A: Authenticate and request session join
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
  A[Participant browser] <-->|SRTP over UDP preferred| M[mahfel-media SFU]
  B[Participant browser] <-->|SRTP over UDP preferred| M
  C[Participant browser] <-->|SRTP over UDP preferred| M
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
