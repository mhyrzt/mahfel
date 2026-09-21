-- Revocable invite links: a short, human-typable bearer capability (ADR 0021).
--
-- One table holds room links and session links so a code resolves
-- deterministically across both. `target_id` is deliberately NOT a foreign key:
-- the target may be a room or a session, and PostgreSQL cannot enforce a
-- polymorphic reference without a trigger. Link rows outlive their target's
-- operational lifetime; resolution looks the target up and fails closed when it
-- is gone.
CREATE TABLE mahfel.invite_links (
    id UUID PRIMARY KEY,
    -- Stored lowercase without separators, and globally unique: the code is the
    -- lookup key, which is why it is plaintext rather than hashed.
    code TEXT NOT NULL UNIQUE CHECK (code ~ '^[a-z]{3}[a-z]{4}[a-z]{3}$'),
    target_kind TEXT NOT NULL CHECK (target_kind IN ('room', 'session')),
    target_id UUID NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked', 'expired')),
    expires_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    CHECK (expires_at > created_at),
    -- Revoked is the only status that carries a timestamp, so a row cannot claim
    -- revocation without recording when.
    CHECK (
        (status = 'revoked' AND revoked_at IS NOT NULL)
        OR (status <> 'revoked' AND revoked_at IS NULL)
    )
);

-- Management reads one target's links, newest first.
CREATE INDEX invite_links_target_idx
ON mahfel.invite_links (target_kind, target_id, created_at DESC);

-- An operator listing by creator; also keeps the actor foreign key covered.
CREATE INDEX invite_links_created_by_idx
ON mahfel.invite_links (created_by, created_at DESC);
