# Collaboration client boundary

`MahfelCollaborationProvider` obtains a single-use WebSocket ticket, connects directly to
`mahfel-api`, exchanges Yjs binary sync messages, and relays ephemeral awareness. Lexical notes
and Excalidraw whiteboard use separate Yjs documents for each session. The REST API owns
permission, shared navigation, presentation, and timeline operations.

PNG and JPEG images embedded in either document use the existing verified session-file upload.
The client registers each ready file with the notes or whiteboard document before publishing its
stable file ID in Yjs. Other clients request fresh authorized download URLs and load the bytes
directly from S3-compatible storage. Image bytes and expiring URLs are never stored in Yjs.
