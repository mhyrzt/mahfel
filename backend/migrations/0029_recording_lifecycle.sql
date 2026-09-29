CREATE TABLE mahfel.recordings (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id),
    status TEXT NOT NULL CHECK (status IN (
        'starting', 'recording', 'paused', 'stopping', 'finalizing', 'ready', 'failed'
    )),
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    capture_generation INTEGER NOT NULL DEFAULT 1 CHECK (capture_generation > 0),
    initiated_by UUID NULL REFERENCES mahfel.actors (id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    stopped_at TIMESTAMPTZ NULL,
    warning TEXT NULL,
    partial_coverage BOOLEAN NOT NULL DEFAULT FALSE,
    manifest_key TEXT NULL,
    CHECK (stopped_at IS NULL OR stopped_at >= started_at)
);
CREATE UNIQUE INDEX recording_one_capture_per_session ON mahfel.recordings (session_id)
WHERE status IN ('starting', 'recording', 'paused', 'stopping');
CREATE INDEX recordings_session_idx ON mahfel.recordings (session_id, started_at DESC, id);

CREATE TABLE mahfel.recording_commands (
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id),
    command_id UUID NOT NULL,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    action_kind TEXT NOT NULL CHECK (action_kind IN ('start', 'pause', 'resume', 'stop')),
    response JSONB NOT NULL,
    PRIMARY KEY (session_id, command_id)
);

CREATE TABLE mahfel.recording_audit (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    recording_id UUID NOT NULL REFERENCES mahfel.recordings (id),
    actor_id UUID NULL REFERENCES mahfel.actors (id),
    action_kind TEXT NOT NULL CHECK (action_kind IN ('start', 'pause', 'resume', 'stop')),
    reason TEXT NOT NULL CHECK (reason IN ('manual', 'auto', 'session_end')),
    revision BIGINT NOT NULL CHECK (revision > 0),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE FUNCTION mahfel.protect_recording_audit() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'recording audit is immutable' USING ERRCODE = '23505';
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER protect_recording_audit
BEFORE UPDATE OR DELETE ON mahfel.recording_audit
FOR EACH ROW EXECUTE FUNCTION mahfel.protect_recording_audit();

CREATE FUNCTION mahfel.record_session_transition() RETURNS TRIGGER AS $$
DECLARE
    recording_id UUID;
    recording_revision BIGINT;
BEGIN
    IF OLD.status = 'scheduled' AND NEW.status = 'live' AND NEW.auto_record THEN
        recording_id := gen_random_uuid();
        INSERT INTO mahfel.recordings (id, session_id, status)
        VALUES (recording_id, NEW.id, 'starting');
        INSERT INTO mahfel.recording_audit (recording_id, action_kind, reason, revision)
        VALUES (recording_id, 'start', 'auto', 1);
    ELSIF OLD.status = 'live' AND NEW.status = 'ended' THEN
        UPDATE mahfel.recordings SET
            status = 'stopping', revision = revision + 1, stopped_at = clock_timestamp()
        WHERE session_id = NEW.id AND status IN ('starting', 'recording', 'paused')
        RETURNING id, revision INTO recording_id, recording_revision;
        IF recording_id IS NOT NULL THEN
            INSERT INTO mahfel.recording_audit (recording_id, action_kind, reason, revision)
            VALUES (recording_id, 'stop', 'session_end', recording_revision);
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER record_session_transition AFTER UPDATE OF status ON mahfel.sessions
FOR EACH ROW EXECUTE FUNCTION mahfel.record_session_transition();
