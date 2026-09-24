CREATE TABLE mahfel.collaboration_documents (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('notes', 'whiteboard')),
    schema_version SMALLINT NOT NULL CHECK (schema_version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, kind)
);

CREATE TABLE mahfel.collaboration_edit_grants (
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id) ON DELETE CASCADE,
    can_edit BOOLEAN NOT NULL,
    updated_by UUID NOT NULL REFERENCES mahfel.actors (id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, actor_id)
);

CREATE TABLE mahfel.collaboration_sequences (
    session_id UUID PRIMARY KEY REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    next_sequence BIGINT NOT NULL DEFAULT 1 CHECK (next_sequence > 0)
);

CREATE TABLE mahfel.collaboration_events (
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    event_sequence BIGINT NOT NULL CHECK (event_sequence > 0),
    id UUID NOT NULL UNIQUE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    document_id UUID NULL REFERENCES mahfel.collaboration_documents (id),
    kind TEXT NOT NULL CHECK (kind IN (
        'document_update', 'viewport', 'presentation', 'edit_grant'
    )),
    update_bytes BYTEA NULL,
    payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, event_sequence),
    CHECK ((kind = 'document_update') = (document_id IS NOT NULL AND update_bytes IS NOT NULL))
);

CREATE INDEX collaboration_events_document_idx
ON mahfel.collaboration_events (document_id, event_sequence)
WHERE document_id IS NOT NULL;

CREATE INDEX collaboration_events_viewport_idx
ON mahfel.collaboration_events (session_id, (payload ->> 'surface'), event_sequence DESC)
WHERE kind = 'viewport';

CREATE TABLE mahfel.collaboration_checkpoints (
    document_id UUID NOT NULL REFERENCES mahfel.collaboration_documents (id) ON DELETE CASCADE,
    through_sequence BIGINT NOT NULL CHECK (through_sequence >= 0),
    update_bytes BYTEA NOT NULL,
    state_vector BYTEA NOT NULL,
    sha_256 BYTEA NOT NULL CHECK (octet_length(sha_256) = 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (document_id, through_sequence)
);

CREATE TABLE mahfel.collaboration_presentations (
    session_id UUID PRIMARY KEY REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    selection_event_id UUID NULL REFERENCES mahfel.collaboration_events (id),
    current_page INTEGER NULL CHECK (current_page > 0),
    total_pages INTEGER NULL CHECK (total_pages > 0),
    CHECK ((selection_event_id IS NULL) = (current_page IS NULL AND total_pages IS NULL)),
    CHECK (current_page IS NULL OR current_page <= total_pages)
);

CREATE TABLE mahfel.collaboration_presentation_assets (
    event_id UUID NOT NULL REFERENCES mahfel.collaboration_events (id) ON DELETE CASCADE,
    ordinal SMALLINT NOT NULL CHECK (ordinal >= 0),
    file_id UUID NOT NULL REFERENCES mahfel.files (id) ON DELETE RESTRICT,
    PRIMARY KEY (event_id, ordinal)
);

CREATE INDEX collaboration_presentation_assets_file_idx
ON mahfel.collaboration_presentation_assets (file_id);

CREATE FUNCTION mahfel.protect_presented_file() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'deleting' AND OLD.status <> 'deleting'
        AND EXISTS (
            SELECT 1 FROM mahfel.collaboration_presentation_assets WHERE file_id = OLD.id
        )
    THEN
        RAISE EXCEPTION 'file is referenced by a presentation' USING ERRCODE = '23505';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER protect_presented_file_before_delete
BEFORE UPDATE OF status ON mahfel.files
FOR EACH ROW EXECUTE FUNCTION mahfel.protect_presented_file();
