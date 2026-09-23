-- One immutable placement decision per session. Redis coordinates the first
-- claim, while this row keeps a session sticky after Redis loses its keys.
CREATE TABLE mahfel.media_placements (
    session_id UUID PRIMARY KEY REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    room_id UUID NOT NULL REFERENCES mahfel.rooms (id),
    node_id TEXT NOT NULL CHECK (node_id ~ '^[A-Za-z0-9_-]{1,64}$'),
    max_rooms INTEGER NOT NULL CHECK (max_rooms > 0),
    active_rooms BIGINT NOT NULL CHECK (active_rooms >= 0),
    reason TEXT NOT NULL,
    decided_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX media_placements_node_decided_idx
ON mahfel.media_placements (node_id, decided_at DESC);
