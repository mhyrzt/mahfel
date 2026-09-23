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
  A->>P: Resolve manage_session for the room
  A->>P: Insert session (scheduled)
  A-->>C: 201 session
  C->>A: POST /v1/sessions/{session_id}/start
  A->>P: Resolve manage_session, then update to live
  A-->>C: 200 session (live)
  C->>A: POST /v1/sessions/{session_id}/end
  A->>P: Require manager membership, then update to ended
  A-->>C: 200 session (ended)
```

A session moves `scheduled` to `live` to `ended`, or `scheduled` to `cancelled`; any other
transition is rejected with `409` and leaves the row unchanged. At most one session per room may be
`live`; a partial unique index makes a concurrent second start fail with `409` instead of creating a
second live session. The optional planned start never gates an explicit start, and the optional
1–10080 minute duration never triggers an automatic end. A session may have a custom title; absent
one, reads display the room's current title. `PATCH /v1/sessions/{session_id}` may change or clear
title and advisory schedule fields in any state, including ended/cancelled, with the
`manage_session` capability. Every accepted action writes a structured `audit` log line. Scheduling
and starting require `manage_session`; end and cancel remain manager-scoped. Member-facing join lives in the session-join flow
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
never appears in logs or metric labels (ADR 0021).

## Invite redemption

```mermaid
sequenceDiagram
  participant B as Holder browser
  participant A as mahfel-api
  participant P as PostgreSQL
  participant R as Redis
  B->>A: POST /v1/invite-links/redeem {code}
  A->>R: Count the attempt (principal and address)
  alt over budget
    A-->>B: 404, identical to an invalid code
  else within budget
    A->>A: Normalize the code
    A->>P: Look up, and resolve the target
    alt unknown, expired, revoked, or target gone
      A-->>B: 404, identical to an invalid code
    else usable
      alt caller already has access
        A-->>B: 200 already_member
      else
        A->>P: Insert a pending request (origin invite_link)
        A-->>B: 200 waiting
        Note over A: A manager approves or rejects
        A->>P: Session request: participant and interval
        A->>P: Room request: durable room membership
      end
    end
  end
```

Redemption resolves and queues; it never joins. Session-link approval grants attendance in that
session only and no membership, so the holder may be asked again later. Room-link approval grants
durable membership and needs no live session, because the room outlives any one session. Every
unusable code returns one identical error and attempts are rate-limited first, so the endpoint cannot
be used to enumerate valid codes.

## Admission

```mermaid
sequenceDiagram
  participant B as Joining browser
  participant M as Manager browser
  participant A as mahfel-api
  participant P as PostgreSQL
  B->>A: join session
  A->>P: Authorize membership or visibility
  A->>A: Resolve effective policy (session override, else room)
  alt policy is open
    A->>P: Insert participant and open interval
    A-->>B: joined
  else policy is lobby
    A->>P: Insert pending admission request
    A-->>B: join_waiting (no participant, no capabilities)
    M->>A: GET /v1/sessions/{id}/lobby
    A-->>M: waiting principals
    M->>A: POST /v1/sessions/{id}/lobby/{actor}/admit
    A->>P: Mark admitted, insert participant, open interval
    A-->>B: admitted
  end
```

Authorization runs first and is unchanged; admission decides only whether an already-authorized
caller enters now or waits. A session's access settings inherit from its room and may only tighten,
so a session cannot expose a room the room itself protects. Ending a session retires any pending
requests, so no one is left waiting on a session that no longer exists.

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
  A->>P: Read existing session placement
  A->>R: Read live media-node heartbeats
  A->>R: Atomically claim node and room slot if unplaced
  A->>P: Insert or read one durable session decision
  A-->>B: Joined with stable participant identity
  B->>A: RTC signaling frames
  A->>R: Verify assigned node still has a fresh heartbeat
  A->>M: Private gRPC on the assigned node
```

## Moderation

```mermaid
sequenceDiagram
  participant B as Manager browser
  participant A as mahfel-api
  participant P as PostgreSQL
  participant M as mahfel-media
  B->>A: mute_participant {actor_id, kind, source}
  A->>P: Resolve membership and capability
  alt no mute_others
    A-->>B: forbidden
  else
    A->>P: Load the active participant
    alt no active participant
      A-->>B: not_found, before any media call
    else
      A->>P: Record muted_at, mute_kind, muted_by
      A->>M: gRPC PauseParticipantSource {room, actor, source}
      alt media plane unreachable
        Note over A: Audited as degraded; the durable mute stands
      end
      A-->>B: participant_muted
      Note over A: Fanned out to every connection of the participant
    end
  end
```

The order is the property that matters, and it is the same for every moderation action: resolve the
capability, resolve the target against the authoritative store, change durable state, reach the media
plane, then audit and propagate. Capability first so an unauthorized caller cannot probe for who
exists, and the state change before enforcement so a media outage degrades rather than misreports.

A capability withdrawal reconciles rather than sweeping: removing a membership or blocking closes
the producers whose source the principal may no longer publish, while a demotion closes nothing
because a member keeps `publish_media` and `share_screen`. Removing a membership while a principal is
publishing closes their producers for the same reason a block does: the grant they were publishing
under is gone.

Presence is derived from the connection registry rather than from a single connection, so a
participant on two devices stays present when one closes and `participant_left` is emitted only at
the last one.

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
