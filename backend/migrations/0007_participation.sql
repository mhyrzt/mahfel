-- Durable participation records: one row per connected stretch.
--
-- This is the generic replacement for an attendance concept. A participant
-- accumulates intervals; the intervals aggregate to connected duration and are
-- corrected by superseding rather than overwriting, so the original
-- measurement and the reason it changed both survive.
CREATE TABLE mahfel.participation_intervals (
    id UUID PRIMARY KEY,
    participant_id UUID NOT NULL
    REFERENCES mahfel.session_participants (id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NULL,
    -- Set when a correction replaced this interval. The row is retained.
    superseded_by UUID NULL REFERENCES mahfel.participation_intervals (id),
    correction_reason TEXT NULL CHECK (
        correction_reason IS NULL
        OR correction_reason IN ('wrong_end', 'wrong_start', 'erroneous', 'manual_adjustment')
    ),
    created_at TIMESTAMPTZ NOT NULL,
    -- A finished interval cannot end before it starts. An open interval is fine.
    CHECK (ended_at IS NULL OR ended_at >= started_at),
    -- A superseded interval must name its replacement and a reason; an active
    -- one must name neither.
    CHECK (
        (
            superseded_by IS NULL
            AND correction_reason IS NULL
        )
        OR (
            superseded_by IS NOT NULL
            AND correction_reason IS NOT NULL
        )
    )
);

-- At most one open interval per participant, so a duplicated connect or a
-- racing join collides instead of recording two open stretches. This is the
-- database half of the "intervals never overlap" invariant; closed intervals
-- are excluded so history is append-only.
CREATE UNIQUE INDEX participation_intervals_one_open_per_participant_idx
ON mahfel.participation_intervals (participant_id) WHERE ended_at IS NULL;

-- Session queries read every interval for a session, newest activity first.
CREATE INDEX participation_intervals_session_id_idx
ON mahfel.participation_intervals (session_id, started_at);

-- Participant queries read that actor's intervals across sessions.
CREATE INDEX participation_intervals_actor_id_idx
ON mahfel.participation_intervals (actor_id, started_at);
