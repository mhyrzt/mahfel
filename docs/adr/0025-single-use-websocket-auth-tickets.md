# ADR 0025: Single-use tickets for browser WebSocket authentication

## Status

Accepted

## Context

Native WebSocket clients can send an Authorization header, but the browser WebSocket API cannot.
Putting a Keycloak access token in browser JavaScript or a WebSocket URL conflicts with ADR 0019's
server-only token boundary. The API already accepts an auth-first-message and shares its verifier
with REST.

## Decision

- An authenticated client requests `POST /v1/ws/tickets`. The API verifies the ordinary bearer
  credential, then returns a random ticket with a ten-second lifetime and `Cache-Control: no-store`.
- Store only a SHA-256 ticket hash and the minimum verified principal needed by WebSocket policy in
  Redis. Atomically consume and delete the ticket during the first WebSocket auth frame. A ticket
  cannot authenticate a second connection.
- The browser obtains the ticket through the exact BFF allowlist entry for this POST route, then
  connects directly to `/v1/ws` and sends the ticket in its auth-first-message. It never receives
  or handles the Keycloak access or refresh token.
- Keep bearer authentication on the upgrade request and bearer-token auth frames for native clients.
  If both credential fields are sent, reject the frame.
- Redis is the ephemeral ticket store. If it is unavailable, ticket issuance fails closed with 503
  and ticket authentication fails closed.
- This ADR supersedes ADR 0019's final bullet about the browser supplying an access token in the
  auth-first-message. All other ADR 0019 decisions remain in force.

## Consequences

Browser signaling remains direct to the API without exposing reusable Keycloak credentials. The API
adds a narrow credential-minting endpoint and needs Redis for browser WebSocket authentication.
Tickets expire quickly, are single-use across API replicas, and contain no reusable provider token.
