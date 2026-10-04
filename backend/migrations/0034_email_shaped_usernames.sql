-- Keycloak usernames may be email-shaped. Match the domain Username validator
-- while preserving the existing length, trim, and whitespace requirements.
ALTER TABLE mahfel.users
DROP CONSTRAINT users_username_valid,
ADD CONSTRAINT users_username_valid CHECK (
    CHAR_LENGTH(username) BETWEEN 1 AND 100
    AND username = BTRIM(username)
    AND username !~ '[[:space:]]'
);

ALTER TABLE mahfel.identities
DROP CONSTRAINT identities_username_valid,
ADD CONSTRAINT identities_username_valid CHECK (
    CHAR_LENGTH(username) BETWEEN 1 AND 100
    AND username = BTRIM(username)
    AND username !~ '[[:space:]]'
);
