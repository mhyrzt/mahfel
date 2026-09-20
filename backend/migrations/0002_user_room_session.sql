CREATE TABLE mahfel.users (
    id UUID PRIMARY KEY,
    display_name TEXT NOT NULL CHECK (char_length(display_name) BETWEEN 1 AND 200),
    status TEXT NOT NULL CHECK (status IN ('active', 'disabled')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at)
);

CREATE TABLE mahfel.rooms (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL CHECK (char_length(title) BETWEEN 1 AND 300),
    created_by UUID NOT NULL REFERENCES mahfel.users (id),
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at)
);

CREATE INDEX rooms_created_by_idx ON mahfel.rooms (created_by);

CREATE TABLE mahfel.sessions (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES mahfel.rooms (id),
    status TEXT NOT NULL CHECK (status IN ('scheduled', 'live', 'ended', 'cancelled')),
    scheduled_start_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at),
    CHECK (ended_at IS NULL OR started_at IS NOT NULL),
    CHECK (ended_at IS NULL OR ended_at >= started_at)
);

CREATE INDEX sessions_room_id_idx
ON mahfel.sessions (room_id);
CREATE INDEX sessions_status_idx
ON mahfel.sessions (status);

-- A room hosts at most one live session at a time. The partial index makes the
-- rule a database invariant, so concurrent starts cannot both win.
CREATE UNIQUE INDEX sessions_one_live_per_room_idx
ON mahfel.sessions (room_id) WHERE status = 'live';
