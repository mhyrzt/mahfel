ALTER TABLE mahfel.users
    ADD COLUMN avatar_url TEXT NULL
        CHECK (avatar_url IS NULL OR char_length(avatar_url) BETWEEN 1 AND 2048);

CREATE TABLE mahfel.identities (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES mahfel.users (id) ON DELETE CASCADE,
    issuer TEXT NOT NULL CHECK (char_length(issuer) BETWEEN 1 AND 500),
    subject TEXT NOT NULL CHECK (char_length(subject) BETWEEN 1 AND 500),
    email TEXT NULL CHECK (email IS NULL OR char_length(email) BETWEEN 3 AND 320),
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    display_name TEXT NULL
        CHECK (display_name IS NULL OR char_length(display_name) BETWEEN 1 AND 200),
    avatar_url TEXT NULL
        CHECK (avatar_url IS NULL OR char_length(avatar_url) BETWEEN 1 AND 2048),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at),
    CONSTRAINT identities_issuer_subject_key UNIQUE (issuer, subject)
);

CREATE INDEX identities_user_id_idx ON mahfel.identities (user_id);

CREATE TABLE mahfel.local_credentials (
    user_id UUID PRIMARY KEY REFERENCES mahfel.users (id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL CHECK (char_length(password_hash) BETWEEN 1 AND 500),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at)
);
