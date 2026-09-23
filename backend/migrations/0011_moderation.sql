-- Mute is a state of a participation, not of a person. Leaving is terminal, so
-- a rejoin creates a new participant row and clears the mute. Keeping someone
-- out after they leave is a block, which is why the two live in different
-- places rather than being degrees of the same thing.
ALTER TABLE mahfel.session_participants
ADD COLUMN muted_at TIMESTAMPTZ NULL,
ADD COLUMN mute_kind TEXT NULL CHECK (mute_kind IN ('temporary', 'permanent')),
ADD COLUMN muted_by UUID NULL REFERENCES mahfel.actors (id);

-- The three mute columns move together. A muted row always records when, how
-- firmly, and by whom; an unmuted row records none of them. This is the same
-- invariant the domain's MuteKind holds in memory, enforced here as defense in
-- depth so a partial write cannot make a mute permanent by omission.
ALTER TABLE mahfel.session_participants
ADD CONSTRAINT session_participants_mute_check
CHECK (
    (muted_at IS NULL AND mute_kind IS NULL AND muted_by IS NULL)
    OR (muted_at IS NOT NULL AND mute_kind IS NOT NULL AND muted_by IS NOT NULL)
);

-- A durable bar on rejoining. Room-scoped and session-scoped blocks are
-- independent facts: a room block denies every session in that room by
-- enforcement, so nothing writes per-session rows for it, and lifting one block
-- therefore never disturbs another.
CREATE TABLE mahfel.blocks (
    id UUID PRIMARY KEY,
    -- Exactly one scope. A room block names the room; a session block names the
    -- session and, through it, the room it belongs to.
    room_id UUID NULL REFERENCES mahfel.rooms (id) ON DELETE CASCADE,
    session_id UUID NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    -- Who imposed it, for the audit trail.
    blocked_by UUID NOT NULL REFERENCES mahfel.actors (id),
    reason TEXT NULL,
    blocked_at TIMESTAMPTZ NOT NULL,
    -- NULL means until explicitly lifted, which is the education case; a
    -- timestamp lets a one-off meeting's block lapse on its own.
    expires_at TIMESTAMPTZ NULL,
    lifted_at TIMESTAMPTZ NULL,
    lifted_by UUID NULL REFERENCES mahfel.actors (id),
    CHECK ((room_id IS NULL) <> (session_id IS NULL)),
    -- An end is recorded by `lifted_at`, whether a principal lifted the block or
    -- it lapsed. `lifted_by` is present only when a principal did it, so an
    -- expiry records nobody: no one made the call.
    CHECK ((lifted_by IS NULL) OR (lifted_at IS NOT NULL)),
    -- A block cannot expire before it was imposed.
    CHECK (expires_at IS NULL OR expires_at > blocked_at)
);

-- At most one un-lifted block per principal and target, so re-blocking an
-- already-blocked principal extends the existing block rather than stacking a
-- second row whose precedence would be undefined.
--
-- A partial index cannot also require `expires_at > now()`, because `now()` is
-- not immutable, so expiry is handled on the write path instead: inserting a
-- block first marks any lapsed block for the same target as ended. That keeps
-- "a lapsed block does not conflict with a new one" true without an
-- unindexable predicate.
CREATE UNIQUE INDEX blocks_one_active_room_idx
ON mahfel.blocks (room_id, actor_id)
WHERE lifted_at IS NULL AND room_id IS NOT NULL;

CREATE UNIQUE INDEX blocks_one_active_session_idx
ON mahfel.blocks (session_id, actor_id)
WHERE lifted_at IS NULL AND session_id IS NOT NULL;

-- The join path asks "is this actor blocked from this room or this session?",
-- so both lookups are indexed on the active set only.
CREATE INDEX blocks_active_room_idx
ON mahfel.blocks (actor_id, room_id)
WHERE lifted_at IS NULL AND room_id IS NOT NULL;

CREATE INDEX blocks_active_session_idx
ON mahfel.blocks (actor_id, session_id)
WHERE lifted_at IS NULL AND session_id IS NOT NULL;
