# Persistent room chat

Each room has one PostgreSQL chat channel across its sessions. Current room members can post and
read messages. A message is either room-wide or scoped to one session: the room conversation lists
only messages with no session scope, and a session conversation lists only that session's messages.
Replies inherit their root message's scope. History lists root messages newest first with an
exclusive `(created_at, id)` cursor; direct replies have a separate paginated timeline. Replies
cannot have replies, and a deleted message cannot receive a new reply. Bodies are nonblank and
limited to 4,000 Unicode characters. Authors may edit or soft-delete for 15 minutes; room managers
may soft-delete any message. Previous bodies remain in the revision audit table.

## Committed events and delivery

PostgreSQL appends an event in the same transaction as each create, edit, delete, or reaction
change. Each channel has a monotonically increasing, gapless sequence. Events preserve the
message snapshot at the time of the change, including a tombstone with no body for deletion.

Authenticated WebSocket clients send `subscribe_chat` with a room ID and last applied sequence.
The API verifies current membership and rejects a cursor beyond the channel head.
`chat_subscribed` acknowledges the subscription. The `chat_event.event` payload carries a sequence and event ID for
client deduplication, and the session scope for the message so a client can filter the conversation
it is viewing. Room chat does not require joining a live session. A client can reconnect to
any API replica and resume from its last applied sequence.

Each API replica reads committed events from PostgreSQL for its local subscribers. Redis pub/sub
wakes replicas promptly after a write; a two-second database poll catches events when a publish
is lost or Redis is unavailable. Membership is rechecked before a batch is delivered. Each
connection has a bounded outbound queue; a full queue drops the connection so it can reconnect
and resume. Application state is recoverable, while individual network frames are not promised
exactly-once delivery. Redis contains no durable chat facts.

## Structured interactions and activity

Each actor can hold one reaction on a message and may replace or remove it. A message can have
different emoji from different actors. Supported reactions are 👍, 👎, ❤️, 😂, 😮, and 😢.
Reaction writes are atomic and emit committed events. Structured mention IDs are validated
against current room membership on creation and edit; at most 20 distinct actors may be
mentioned. Mention notifications are postponed.

The per-user read marker is a PostgreSQL cursor over channel events. The API exposes only the
caller's marker. It moves forward only and cannot exceed the committed event head. Typing and
presence are Redis TTL aggregates for a room: online actors are refreshed every 15 seconds and
expire after 45 seconds; typing expires after five seconds without refresh. `chat_activity`
WebSocket frames contain aggregate online and typing counts, not actor identifiers. The
aggregate may take up to one TTL to decay after an abrupt disconnect.

## Attachments and abuse bounds

The file context links an upload to a chat message. The composer collects attachments while a
message is written and uploads them against the message once it exists; sent messages do not accept
new attachments. Message history and reply pages include each message's ready attachments so
clients render media albums and file cards inline. Uploads remain pending or quarantined until
verified; only ready files appear in file listings and are downloadable. The file adapter
rechecks that the message is live and the owner is a current room member when metadata is
created and when a verified file is published. At most four pending, quarantined, or ready files
may reference one message. The global file contract limits each object to 50 MiB. Soft deletion
marks linked files for worker cleanup.

Publishing a verified attachment appends an `edited` message snapshot in the same PostgreSQL
transaction as the transition to ready, without changing the message's body-edit timestamp.
WebSocket snapshots include additive `attachment_ids` metadata. Clients fetch at most four ready
files for the affected message instead of reloading conversation history; repeated publication
does not append another event.

Redis counters limit each actor to ten new messages or replies per room per ten seconds and 60
per minute, across API replicas. Redis failure rejects new posts while the limit cannot be
enforced. Managers can moderate abusive content with the existing soft-delete permission.
