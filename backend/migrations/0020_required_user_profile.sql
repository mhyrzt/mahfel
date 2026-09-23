-- User profiles require a valid primary email and username. This migration
-- backfills existing rows deterministically before applying the constraints.
ALTER TABLE mahfel.users
ADD COLUMN email TEXT,
ADD COLUMN username TEXT;

UPDATE mahfel.users AS user_row
SET email = COALESCE(
    (
        SELECT identities.email
        FROM mahfel.identities AS identities
        WHERE identities.user_id = user_row.id AND identities.email IS NOT NULL
        ORDER BY identities.created_at, identities.id
        LIMIT 1
    ),
    'user-' || REPLACE(user_row.id::TEXT, '-', '') || '@users.mahfel.invalid'
),
username = COALESCE(
    (
        SELECT 'u_' || REPLACE(identities.id::TEXT, '-', '')
        FROM mahfel.identities AS identities
        WHERE identities.user_id = user_row.id
        ORDER BY identities.created_at, identities.id
        LIMIT 1
    ),
    'u_' || REPLACE(user_row.id::TEXT, '-', '')
);

ALTER TABLE mahfel.users
ALTER COLUMN email SET NOT NULL,
ALTER COLUMN username SET NOT NULL,
ADD CONSTRAINT users_email_valid CHECK (
    CHAR_LENGTH(email) BETWEEN 3 AND 320
    AND POSITION('@' IN email) > 1
    AND POSITION('@' IN email) < CHAR_LENGTH(email)
    AND email !~ '[[:space:]]'
),
ADD CONSTRAINT users_username_valid CHECK (
    CHAR_LENGTH(username) BETWEEN 1 AND 100
    AND username = BTRIM(username)
    AND username !~ '[[:space:]@]'
);

ALTER TABLE mahfel.identities
ADD COLUMN username TEXT;

UPDATE mahfel.identities
SET username = 'u_' || REPLACE(id::TEXT, '-', '');

ALTER TABLE mahfel.identities
ALTER COLUMN username SET NOT NULL,
ADD CONSTRAINT identities_username_valid CHECK (
    CHAR_LENGTH(username) BETWEEN 1 AND 100
    AND username = BTRIM(username)
    AND username !~ '[[:space:]@]'
);
