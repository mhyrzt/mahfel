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

CREATE TABLE mahfel.classroom_sessions (
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

CREATE INDEX classroom_sessions_room_id_idx
ON mahfel.classroom_sessions (room_id);
CREATE INDEX classroom_sessions_status_idx
ON mahfel.classroom_sessions (status);
