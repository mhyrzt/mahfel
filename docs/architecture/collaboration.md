# Session collaboration

Each session has one versioned notes document and one versioned whiteboard document. The API
creates their identities on first authorized access. PostgreSQL owns document metadata, edit
grants, the ordered event journal, and checkpoints. Redis carries only ephemeral awareness between
API replicas. Yjs runs in the browser; Yrs validates and restores document updates in the API.

The browser opens `/v1/ws/collaboration/{session_id}/{kind}` with a single-use ticket as the
first text frame. Subsequent document sync and awareness frames use Yjs protocol v1 binary
messages. The API caps frames at 1 MiB and 100 frames per second per connection. It polls the
ordered PostgreSQL journal to converge clients connected to different replicas. Notes use
Lexical's Yjs adapter. The whiteboard stores Excalidraw elements by ID in a Yjs map; deleted
elements remain tombstoned in the scene model.

Only an admitted current room member or a room manager may read a live session document. A
manager can always edit and can grant or revoke a single shared edit permission for an admitted participant. A
granted editor may also present files. Any admitted participant may move the shared notes scroll
or whiteboard viewport. After the session ends, current room members can inspect the workspace
read only. Rejected access does not expose whether a document exists.

The journal assigns one increasing sequence per session to document updates, viewport moves,
presentation changes, and edit grants. Checkpoints contain Yrs state, state vector, sequence,
and SHA-256 digest. An invalid checkpoint is ignored and the retained journal is replayed.
Historical updates remain available for a later event player and recording alignment; awareness
is never persisted. The REST event endpoint exposes pages of at most 100 events, with binary
document updates encoded in base64.

Presenters choose one ready PDF or up to 50 ordered ready images from the session or its room.
The API reads the PDF to determine page count, validates the file context and permission, and
records selection, page moves, and stop actions. PostgreSQL retains references to selected files
and prevents their deletion while the presentation history references them. Each viewer obtains
a fresh signed download and renders the current slide locally. Presentation changes are distinct
from whiteboard drawing changes.

The current browser workspace provides live editing and read only access to ended sessions. It
does not yet provide a historical scrubber across the journal; M9's player can consume that
timeline alongside recorded audio and video.
