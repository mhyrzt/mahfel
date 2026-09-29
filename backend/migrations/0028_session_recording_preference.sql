ALTER TABLE mahfel.sessions
ADD COLUMN auto_record BOOLEAN NOT NULL DEFAULT TRUE;

CREATE FUNCTION mahfel.protect_session_recording_preference() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status <> 'scheduled' AND NEW.auto_record IS DISTINCT FROM OLD.auto_record THEN
        RAISE EXCEPTION 'recording preference is locked' USING ERRCODE = '23505';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER protect_session_recording_preference
BEFORE UPDATE OF auto_record ON mahfel.sessions
FOR EACH ROW EXECUTE FUNCTION mahfel.protect_session_recording_preference();
