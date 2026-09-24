# Collaboration client boundary

`MahfelCollaborationProvider` obtains a single-use WebSocket ticket, connects directly to
`mahfel-api`, exchanges Yjs binary sync messages, and relays ephemeral awareness. Lexical notes
and Excalidraw whiteboard use separate Yjs documents for each session. The REST API owns
permission, shared navigation, presentation, and timeline operations.
