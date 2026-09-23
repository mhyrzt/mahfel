# WebSocket client boundary

Browser WebSocket traffic connects directly to `mahfel-api`. Generate the client and server
message types from `specs/asyncapi.json` with `just frontend generate-api` after exporting the
backend specs. The generated types live in `lib/websocket/generated/messages.ts`; the WebSocket
client uses them for outgoing signaling frames. They provide compile-time checks, not runtime
validation of incoming frames.
