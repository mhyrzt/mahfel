CREATE TABLE mahfel.chat_channels (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL UNIQUE
    REFERENCES mahfel.rooms (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE mahfel.chat_messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL
    REFERENCES mahfel.chat_channels (id) ON DELETE CASCADE,
    author_actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    body TEXT NOT NULL CHECK (length(btrim(body)) > 0),
    created_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CHECK (deleted_at IS NULL OR deleted_at >= created_at)
);

CREATE INDEX chat_messages_channel_timeline_idx
ON mahfel.chat_messages (channel_id, created_at, id);

CREATE INDEX chat_messages_author_timeline_idx
ON mahfel.chat_messages (author_actor_id, created_at DESC);
