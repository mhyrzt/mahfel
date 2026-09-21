-- Durable participant identity: one row per actor's presence in one session.
--
-- Keyed by (session_id, actor_id) rather than by transport, so a reconnect or a
-- second device reuses the row. Connection count and connection ordering are
-- ephemeral and are not stored here.
CREATE TABLE mahfel.session_participants (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('active', 'left', 'removed')),
    joined_at TIMESTAMPTZ NOT NULL,
    left_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at),
    CHECK (left_at IS NULL OR left_at >= joined_at)
);

-- At most one active participant per (session, actor), so a reconnect, a second
-- device, or a racing join collides instead of duplicating the identity.
-- Terminal rows are excluded, so history is append-only and the same actor may
-- participate again later.
CREATE UNIQUE INDEX participants_one_active_per_session_actor_idx
ON mahfel.session_participants (session_id, actor_id) WHERE status = 'active';

CREATE INDEX session_participants_session_id_idx
ON mahfel.session_participants (session_id);

CREATE INDEX session_participants_actor_id_idx
ON mahfel.session_participants (actor_id);
