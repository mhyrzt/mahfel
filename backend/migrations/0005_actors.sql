-- Unified actor model: users and service principals share one id space so a
-- room has a single owner column and a single membership column.
CREATE TABLE mahfel.actors (
    id UUID PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('user', 'service')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at)
);

-- Backfill one actor per existing user, copying the user's timestamps.
INSERT INTO mahfel.actors (id, kind, created_at, updated_at)
SELECT
    id,
    'user' AS kind,
    created_at,
    updated_at
FROM mahfel.users;

-- Every user row must have a matching actor row.
ALTER TABLE mahfel.users
ADD CONSTRAINT users_id_actor_fkey
FOREIGN KEY (id) REFERENCES mahfel.actors (id) ON DELETE CASCADE;

CREATE TABLE mahfel.service_principals (
    id UUID PRIMARY KEY REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    -- `name` is a legitimate column name here; RF04 flags it as a keyword.
    name TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200), -- noqa: RF04
    display_name TEXT NULL
    CHECK (display_name IS NULL OR char_length(display_name) BETWEEN 1 AND 200),
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at),
    CONSTRAINT service_principals_name_key UNIQUE (name)
);

CREATE TABLE mahfel.service_api_keys (
    id UUID PRIMARY KEY,
    service_principal_id UUID NOT NULL
    REFERENCES mahfel.service_principals (id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    last_used_at TIMESTAMPTZ NULL,
    CHECK (updated_at >= created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

CREATE INDEX service_api_keys_service_principal_id_idx
ON mahfel.service_api_keys (service_principal_id);

-- A room may now be owned by any actor, not only a user.
ALTER TABLE mahfel.rooms
DROP CONSTRAINT rooms_created_by_fkey;

ALTER TABLE mahfel.rooms
ADD CONSTRAINT rooms_created_by_fkey
FOREIGN KEY (created_by) REFERENCES mahfel.actors (id);

-- Membership is keyed by actor, so a service principal can be a manager or a
-- member exactly like a user. The (room_id, actor_id) primary key is kept.
ALTER TABLE mahfel.room_memberships
RENAME COLUMN user_id TO actor_id;

ALTER TABLE mahfel.room_memberships
DROP CONSTRAINT room_memberships_user_id_fkey;

ALTER TABLE mahfel.room_memberships
ADD CONSTRAINT room_memberships_actor_id_fkey
FOREIGN KEY (actor_id) REFERENCES mahfel.actors (id) ON DELETE CASCADE;

ALTER INDEX mahfel.room_memberships_user_id_idx
RENAME TO room_memberships_actor_id_idx;
