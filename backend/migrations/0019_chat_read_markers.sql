CREATE TABLE mahfel.chat_read_markers (
    channel_id UUID NOT NULL REFERENCES mahfel.chat_channels (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    event_seq BIGINT NOT NULL CHECK (event_seq >= 0),
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (channel_id, actor_id)
);
