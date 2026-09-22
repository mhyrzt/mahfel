-- Admission control: the generic form of a waiting room (ADR 0020).
--
-- A room carries an admission policy. Under 'lobby' an authorized principal
-- becomes a pending request that a capable principal must decide. Admission
-- never replaces authorization; it layers on top of it.
ALTER TABLE mahfel.rooms
ADD COLUMN admission TEXT NOT NULL DEFAULT 'open'
CHECK (admission IN ('open', 'lobby'));

-- Session-level overrides. NULL means "inherit the room", which is why these
-- columns are nullable while the room's policy is not. An override may only
-- tighten access, so a session never exposes a room the room itself protects;
-- the domain enforces that rule and the database holds the values.
ALTER TABLE mahfel.sessions
ADD COLUMN visibility TEXT NULL CHECK (visibility IN ('public', 'invite_only')),
ADD COLUMN admission TEXT NULL CHECK (admission IN ('open', 'lobby'));

CREATE TABLE mahfel.admission_requests (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (
        status IN ('pending', 'admitted', 'rejected', 'expired')
    ),
    -- Where the request came from, so an operator can tell an explicit join
    -- from an invite-link redemption (ADR 0021).
    origin TEXT NOT NULL CHECK (origin IN ('join', 'invite_link')),
    requested_at TIMESTAMPTZ NOT NULL,
    decided_at TIMESTAMPTZ NULL,
    decided_by UUID NULL REFERENCES mahfel.actors (id),
    -- A pending request has no decision; a decided one records when, and a
    -- human decision also records who. An expiry records no principal, because
    -- nobody made the call.
    CHECK (
        (status = 'pending' AND decided_at IS NULL AND decided_by IS NULL)
        OR (
            status = 'expired' AND decided_at IS NOT NULL
            AND decided_by IS NULL
        )
        OR (
            status IN ('admitted', 'rejected') AND decided_at IS NOT NULL
            AND decided_by IS NOT NULL
        )
    ),
    CHECK (decided_at IS NULL OR decided_at >= requested_at)
);

-- At most one outstanding request per (session, actor), mirroring the
-- participants partial index, so a duplicated join cannot queue twice.
CREATE UNIQUE INDEX admission_requests_one_pending_idx
ON mahfel.admission_requests (session_id, actor_id) WHERE status = 'pending';

-- A manager reads one session's queue, oldest first.
CREATE INDEX admission_requests_session_pending_idx
ON mahfel.admission_requests (session_id, requested_at) WHERE status = 'pending';

CREATE INDEX admission_requests_actor_id_idx
ON mahfel.admission_requests (actor_id, requested_at);
