# WebSocket client boundary

Browser WebSocket traffic connects directly to `mahfel-api`. The browser first requests a
single-use, ten-second ticket from the exact `POST /v1/ws/tickets` BFF route. The server attaches
the Keycloak token to that REST request, while browser JavaScript receives only the ticket. The
client sends the ticket in its first `auth` frame; reconnects request a fresh ticket. It never puts
credentials in the WebSocket URL.

Generate the client and server message types from `specs/asyncapi.json` with
`just frontend generate-api` after exporting the backend specs. The generated types live in
`lib/websocket/generated/messages.ts`. They provide compile-time checks, not runtime validation of
incoming frames.
