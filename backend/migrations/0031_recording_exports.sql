CREATE TABLE mahfel.recording_exports (
    id UUID PRIMARY KEY,
    recording_id UUID NOT NULL REFERENCES mahfel.recordings (id),
    manifest_key TEXT NOT NULL,
    profile_version INTEGER NOT NULL DEFAULT 1 CHECK (profile_version > 0),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    job_id UUID NOT NULL REFERENCES mahfel.background_jobs (id),
    status TEXT NOT NULL DEFAULT 'queued' CHECK (
        status IN ('queued', 'rendering', 'ready', 'failed')
    ),
    progress INTEGER NOT NULL DEFAULT 0 CHECK (progress BETWEEN 0 AND 100),
    object_key TEXT NULL,
    sha_256 TEXT NULL,
    duration_ms BIGINT NULL CHECK (duration_ms >= 0),
    size_bytes BIGINT NULL CHECK (size_bytes > 0),
    failure_code TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (recording_id, manifest_key, profile_version),
    CHECK (
        status <> 'ready'
        OR (
            object_key IS NOT NULL
            AND sha_256 IS NOT NULL
            AND duration_ms IS NOT NULL
            AND size_bytes IS NOT NULL
        )
    )
);
CREATE INDEX recording_export_job_idx ON mahfel.recording_exports (job_id);

CREATE TABLE mahfel.recording_export_attempts (
    claim_token UUID PRIMARY KEY,
    export_id UUID NOT NULL REFERENCES mahfel.recording_exports (id),
    object_key TEXT NOT NULL UNIQUE,
    retired BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE FUNCTION mahfel.recording_presentation_anchor() RETURNS TRIGGER AS $$
DECLARE
    anchor_time TIMESTAMPTZ;
    origin_time TIMESTAMPTZ;
BEGIN
    IF NEW.snapshot_kind = 'presentation' AND NEW.payload ->> 'selection_event_id' IS NOT NULL THEN
        SELECT started_at INTO origin_time FROM mahfel.sessions
        WHERE id = (SELECT session_id FROM mahfel.recordings WHERE id = NEW.recording_id);
        SELECT created_at INTO anchor_time FROM mahfel.collaboration_events
        WHERE session_id = (NEW.payload ->> 'session_id')::UUID
            AND kind = 'presentation' AND payload ->> 'action' = 'playback'
            AND event_sequence >= (SELECT event_sequence FROM mahfel.collaboration_events
                WHERE id = (NEW.payload ->> 'selection_event_id')::UUID)
        ORDER BY event_sequence DESC LIMIT 1;
        IF anchor_time IS NOT NULL THEN
            NEW.payload = NEW.payload || jsonb_build_object('playback_anchor_ns',
                (extract(epoch FROM anchor_time - origin_time) * 1000000000)::NUMERIC(20, 0)::TEXT);
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER recording_presentation_anchor BEFORE INSERT ON mahfel.recording_snapshots
FOR EACH ROW EXECUTE FUNCTION mahfel.recording_presentation_anchor();
