CREATE TABLE mahfel.files (
    id UUID PRIMARY KEY,
    object_key TEXT NOT NULL UNIQUE CHECK (char_length(object_key) BETWEEN 1 AND 256),
    owner_actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    room_id UUID NULL REFERENCES mahfel.rooms (id),
    session_id UUID NULL REFERENCES mahfel.sessions (id),
    chat_message_id UUID NULL REFERENCES mahfel.chat_messages (id),
    avatar_actor_id UUID NULL REFERENCES mahfel.actors (id),
    recording_session_id UUID NULL REFERENCES mahfel.sessions (id),
    CHECK (
        num_nonnulls(room_id, session_id, chat_message_id, avatar_actor_id, recording_session_id)
        = 1
    ),
    status TEXT NOT NULL CHECK (
        status IN ('pending', 'quarantined', 'ready', 'rejected', 'deleting')
    ),
    file_name TEXT NOT NULL CHECK (char_length(file_name) BETWEEN 1 AND 255),
    declared_mime TEXT NOT NULL CHECK (char_length(declared_mime) BETWEEN 1 AND 100),
    verified_mime TEXT NULL CHECK (char_length(verified_mime) BETWEEN 1 AND 100),
    size_bytes BIGINT NOT NULL CHECK (size_bytes BETWEEN 1 AND 52428800),
    checksum_sha_256 TEXT NOT NULL
    CHECK (
        char_length(checksum_sha_256) = 44 AND octet_length(decode(checksum_sha_256, 'base64')) = 32
    ),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (updated_at >= created_at),
    CHECK (status <> 'ready' OR verified_mime IS NOT NULL)
);

CREATE INDEX files_room_idx ON mahfel.files (room_id, created_at DESC) WHERE status = 'ready';
CREATE INDEX files_session_idx ON mahfel.files (session_id, created_at DESC) WHERE status = 'ready';
CREATE INDEX files_chat_idx ON mahfel.files (chat_message_id, created_at DESC) WHERE status
= 'ready';
CREATE INDEX files_avatar_idx ON mahfel.files (avatar_actor_id, created_at DESC) WHERE status
= 'ready';
CREATE INDEX files_recording_idx ON mahfel.files (
    recording_session_id, created_at DESC
) WHERE status
= 'ready';
CREATE INDEX files_pending_idx ON mahfel.files (updated_at) WHERE status IN (
    'pending', 'quarantined', 'rejected', 'deleting'
);

CREATE TABLE mahfel.file_download_audit (
    id UUID PRIMARY KEY,
    file_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX file_download_audit_file_idx ON mahfel.file_download_audit (file_id, created_at DESC);
