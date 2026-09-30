ALTER TABLE mahfel.recordings ADD COLUMN deleted_at TIMESTAMPTZ NULL;
ALTER TABLE mahfel.recordings ADD COLUMN cleanup_checked_at TIMESTAMPTZ NULL;
ALTER TABLE mahfel.recording_audit DROP CONSTRAINT recording_audit_action_kind_check;
ALTER TABLE mahfel.recording_audit ADD CONSTRAINT recording_audit_action_kind_check
CHECK (action_kind IN ('start', 'pause', 'resume', 'stop', 'delete'));

CREATE FUNCTION mahfel.protect_deleted_recording() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.deleted_at IS NOT NULL AND (
        NEW.deleted_at IS DISTINCT FROM OLD.deleted_at
        OR NEW.status IS DISTINCT FROM OLD.status
        OR NEW.manifest_key IS DISTINCT FROM OLD.manifest_key
        OR NEW.capture_generation IS DISTINCT FROM OLD.capture_generation
    ) THEN
        RAISE EXCEPTION 'deleted recording cannot be restored' USING ERRCODE = '23505';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER protect_deleted_recording BEFORE UPDATE ON mahfel.recordings
FOR EACH ROW EXECUTE FUNCTION mahfel.protect_deleted_recording();
CREATE INDEX recording_cleanup_idx ON mahfel.recordings (cleanup_checked_at, id)
WHERE deleted_at IS NOT NULL;
