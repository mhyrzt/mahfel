CREATE TABLE mahfel.recording_generations (
    recording_id UUID NOT NULL REFERENCES mahfel.recordings (id),
    generation INTEGER NOT NULL CHECK (generation > 0),
    started_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    ended_at TIMESTAMPTZ NULL,
    boot_id UUID NULL,
    unix_origin_ns NUMERIC(20, 0) NULL,
    receipt_prefix TEXT NULL,
    sealed BOOLEAN NOT NULL DEFAULT FALSE,
    loss_reason TEXT NULL,
    PRIMARY KEY (recording_id, generation)
);
CREATE TABLE mahfel.recording_events (
    recording_id UUID NOT NULL REFERENCES mahfel.recordings (id),
    generation INTEGER NOT NULL,
    event_kind TEXT NOT NULL,
    source_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    payload JSONB NOT NULL,
    PRIMARY KEY (recording_id, event_kind, source_id),
    FOREIGN KEY (recording_id, generation)
    REFERENCES mahfel.recording_generations (recording_id, generation)
);
CREATE TABLE mahfel.recording_snapshots (
    recording_id UUID NOT NULL,
    generation INTEGER NOT NULL,
    snapshot_kind TEXT NOT NULL,
    source_id UUID NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (recording_id, generation, snapshot_kind, source_id),
    FOREIGN KEY (recording_id, generation)
    REFERENCES mahfel.recording_generations (recording_id, generation)
);
CREATE TABLE mahfel.recording_fragments (
    recording_id UUID NOT NULL,
    generation INTEGER NOT NULL,
    object_key TEXT NOT NULL,
    receipt JSONB NOT NULL,
    PRIMARY KEY (recording_id, object_key),
    FOREIGN KEY (recording_id, generation)
    REFERENCES mahfel.recording_generations (recording_id, generation)
);
CREATE TABLE mahfel.recording_assets (
    recording_id UUID NOT NULL REFERENCES mahfel.recordings (id),
    file_id UUID NOT NULL,
    source_key TEXT NOT NULL,
    archive_key TEXT NOT NULL,
    checksum_sha_256 TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    mime TEXT NULL,
    copy_status TEXT NOT NULL DEFAULT 'pending' CHECK (
        copy_status IN ('pending', 'ready', 'failed')
    ),
    PRIMARY KEY (recording_id, file_id)
);
CREATE INDEX recording_asset_pins_idx ON mahfel.recording_assets (file_id)
WHERE copy_status = 'pending';

CREATE FUNCTION mahfel.pin_recording_asset(recording UUID, file_uuid UUID) RETURNS VOID AS $$
BEGIN
    INSERT INTO mahfel.recording_assets (
        recording_id, file_id, source_key, archive_key, checksum_sha_256, size_bytes, mime
    )
    SELECT recording, id, object_key, 'recordings/' || recording || '/assets/' || id,
        checksum_sha_256, size_bytes, verified_mime
    FROM mahfel.files WHERE id = file_uuid AND status = 'ready'
    FOR SHARE
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION mahfel.snapshot_recording_workspace() RETURNS TRIGGER AS $$
DECLARE
    file_uuid UUID;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF NEW.status IN ('paused', 'stopping') AND NEW.status IS DISTINCT FROM OLD.status THEN
            UPDATE mahfel.recording_generations SET ended_at = clock_timestamp()
            WHERE recording_id = NEW.id AND generation = OLD.capture_generation AND ended_at IS NULL;
        END IF;
        IF NEW.capture_generation = OLD.capture_generation THEN
            RETURN NEW;
        END IF;
    END IF;
    INSERT INTO mahfel.recording_generations (recording_id, generation)
    VALUES (NEW.id, NEW.capture_generation);
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'document', document.id, to_jsonb(document)
    FROM mahfel.collaboration_documents AS document WHERE document.session_id = NEW.session_id;
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'checkpoint', latest.document_id, to_jsonb(latest)
    FROM (
        SELECT DISTINCT ON (checkpoint.document_id) checkpoint.*
        FROM mahfel.collaboration_checkpoints AS checkpoint
        JOIN mahfel.collaboration_documents AS document ON document.id = checkpoint.document_id
        WHERE document.session_id = NEW.session_id
        ORDER BY checkpoint.document_id, checkpoint.through_sequence DESC
    ) AS latest;
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'document_update', event.id, to_jsonb(event)
    FROM mahfel.collaboration_events AS event WHERE event.session_id = NEW.session_id
        AND event.kind = 'document_update'
        AND event.event_sequence > COALESCE((
            SELECT max(checkpoint.through_sequence) FROM mahfel.collaboration_checkpoints AS checkpoint
            WHERE checkpoint.document_id = event.document_id
        ), 0);
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'presentation', NEW.session_id, to_jsonb(presentation)
    FROM mahfel.collaboration_presentations AS presentation WHERE presentation.session_id = NEW.session_id;
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'viewport', latest.id, to_jsonb(latest)
    FROM (
        SELECT DISTINCT ON (event.payload ->> 'surface') event.*
        FROM mahfel.collaboration_events AS event WHERE event.session_id = NEW.session_id AND event.kind = 'viewport'
        ORDER BY event.payload ->> 'surface', event.event_sequence DESC
    ) AS latest;
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'edit_grant', grant_state.actor_id, to_jsonb(grant_state)
    FROM mahfel.collaboration_edit_grants AS grant_state WHERE grant_state.session_id = NEW.session_id;
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'presentation_asset', asset.file_id, to_jsonb(asset)
    FROM mahfel.collaboration_presentation_assets AS asset
    JOIN mahfel.collaboration_presentations AS presentation ON presentation.selection_event_id = asset.event_id
    WHERE presentation.session_id = NEW.session_id;
    INSERT INTO mahfel.recording_snapshots
    SELECT NEW.id, NEW.capture_generation, 'document_asset', asset.file_id,
        jsonb_build_object('file_id', asset.file_id, 'document_ids', jsonb_agg(asset.document_id ORDER BY asset.document_id))
    FROM mahfel.collaboration_document_assets AS asset
    JOIN mahfel.collaboration_documents AS document ON document.id = asset.document_id
    WHERE document.session_id = NEW.session_id GROUP BY asset.file_id;
    FOR file_uuid IN
        SELECT asset.file_id FROM mahfel.collaboration_document_assets AS asset
        JOIN mahfel.collaboration_documents AS document ON document.id = asset.document_id
        WHERE document.session_id = NEW.session_id
        UNION
        SELECT asset.file_id FROM mahfel.collaboration_presentation_assets AS asset
        JOIN mahfel.collaboration_presentations AS presentation ON presentation.selection_event_id = asset.event_id
        WHERE presentation.session_id = NEW.session_id
        ORDER BY file_id
    LOOP
        PERFORM mahfel.pin_recording_asset(NEW.id, file_uuid);
    END LOOP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER snapshot_recording_workspace AFTER INSERT OR UPDATE ON mahfel.recordings
FOR EACH ROW EXECUTE FUNCTION mahfel.snapshot_recording_workspace();

CREATE FUNCTION mahfel.capture_collaboration_event() RETURNS TRIGGER AS $$
DECLARE
    recording UUID;
    generation_number INTEGER;
BEGIN
    PERFORM id FROM mahfel.sessions WHERE id = NEW.session_id FOR UPDATE;
    SELECT id, capture_generation INTO recording, generation_number FROM mahfel.recordings
    WHERE session_id = NEW.session_id AND status IN ('starting', 'recording');
    IF recording IS NOT NULL THEN
        INSERT INTO mahfel.recording_events (recording_id, generation, event_kind, source_id, payload)
        VALUES (recording, generation_number, 'collaboration', NEW.id, to_jsonb(NEW));
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER capture_collaboration_event AFTER INSERT ON mahfel.collaboration_events
FOR EACH ROW EXECUTE FUNCTION mahfel.capture_collaboration_event();

CREATE FUNCTION mahfel.lock_chat_recording_boundary() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.session_id IS NOT NULL THEN
        PERFORM id FROM mahfel.sessions WHERE id = NEW.session_id FOR UPDATE;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER lock_chat_recording_boundary BEFORE INSERT OR UPDATE ON mahfel.chat_messages
FOR EACH ROW EXECUTE FUNCTION mahfel.lock_chat_recording_boundary();

CREATE FUNCTION mahfel.capture_chat_event() RETURNS TRIGGER AS $$
DECLARE
    session_uuid UUID;
    recording UUID;
    generation_number INTEGER;
BEGIN
    SELECT session_id INTO session_uuid FROM mahfel.chat_messages WHERE id = NEW.message_id;
    SELECT id, capture_generation INTO recording, generation_number FROM mahfel.recordings
    WHERE session_id = session_uuid AND status IN ('starting', 'recording');
    IF recording IS NOT NULL AND (
        NEW.kind = 'created' OR EXISTS (
            SELECT 1 FROM mahfel.recording_events WHERE recording_id = recording
                AND event_kind = 'chat' AND payload ->> 'message_id' = NEW.message_id::TEXT
                AND payload ->> 'kind' = 'created'
        )
    ) THEN
        INSERT INTO mahfel.recording_events (recording_id, generation, event_kind, source_id, payload)
        VALUES (recording, generation_number, 'chat', NEW.id, to_jsonb(NEW));
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER capture_chat_event AFTER INSERT ON mahfel.chat_events
FOR EACH ROW EXECUTE FUNCTION mahfel.capture_chat_event();

CREATE FUNCTION mahfel.pin_collaboration_recording_asset() RETURNS TRIGGER AS $$
DECLARE
    session_uuid UUID;
    recording UUID;
BEGIN
    IF TG_TABLE_NAME = 'collaboration_document_assets' THEN
        SELECT session_id INTO session_uuid FROM mahfel.collaboration_documents WHERE id = NEW.document_id;
    ELSE
        SELECT session_id INTO session_uuid FROM mahfel.collaboration_events WHERE id = NEW.event_id;
    END IF;
    SELECT id INTO recording FROM mahfel.recordings WHERE session_id = session_uuid AND status IN ('starting', 'recording');
    IF recording IS NOT NULL THEN
        PERFORM mahfel.pin_recording_asset(recording, NEW.file_id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER pin_recording_document_asset AFTER INSERT ON mahfel.collaboration_document_assets
FOR EACH ROW EXECUTE FUNCTION mahfel.pin_collaboration_recording_asset();
CREATE TRIGGER pin_recording_presentation_asset
AFTER INSERT ON mahfel.collaboration_presentation_assets
FOR EACH ROW EXECUTE FUNCTION mahfel.pin_collaboration_recording_asset();

CREATE FUNCTION mahfel.pin_recorded_chat_attachment() RETURNS TRIGGER AS $$
DECLARE
    recording UUID;
BEGIN
    IF NEW.status = 'ready' AND NEW.chat_message_id IS NOT NULL THEN
        FOR recording IN
            SELECT DISTINCT recording_id FROM mahfel.recording_events
            WHERE event_kind = 'chat' AND payload ->> 'message_id' = NEW.chat_message_id::TEXT
        LOOP
            PERFORM mahfel.pin_recording_asset(recording, NEW.id);
        END LOOP;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER pin_recorded_chat_attachment AFTER INSERT OR UPDATE OF status ON mahfel.files
FOR EACH ROW EXECUTE FUNCTION mahfel.pin_recorded_chat_attachment();

CREATE FUNCTION mahfel.protect_pending_recording_asset() RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM mahfel.recording_assets WHERE file_id = OLD.id AND copy_status = 'pending'
    ) THEN
        RAISE EXCEPTION 'file is pinned by a pending recording archive' USING ERRCODE = '23505';
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER protect_pending_recording_asset BEFORE DELETE ON mahfel.files
FOR EACH ROW EXECUTE FUNCTION mahfel.protect_pending_recording_asset();

CREATE FUNCTION mahfel.capture_recording_document() RETURNS TRIGGER AS $$
DECLARE
    recording UUID;
    generation_number INTEGER;
BEGIN
    PERFORM id FROM mahfel.sessions WHERE id = NEW.session_id FOR UPDATE;
    SELECT id, capture_generation INTO recording, generation_number FROM mahfel.recordings
    WHERE session_id = NEW.session_id AND status IN ('starting', 'recording');
    IF recording IS NOT NULL THEN
        INSERT INTO mahfel.recording_snapshots
        VALUES (recording, generation_number, 'document', NEW.id, to_jsonb(NEW))
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER capture_recording_document AFTER INSERT ON mahfel.collaboration_documents
FOR EACH ROW EXECUTE FUNCTION mahfel.capture_recording_document();
