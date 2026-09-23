ALTER TABLE mahfel.chat_channels
ADD COLUMN event_sequence BIGINT NOT NULL DEFAULT 0
CHECK (event_sequence >= 0);

CREATE TABLE mahfel.chat_events (
    channel_id UUID NOT NULL REFERENCES mahfel.chat_channels (id) ON DELETE CASCADE,
    event_seq BIGINT NOT NULL CHECK (event_seq > 0),
    id UUID NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('created', 'edited', 'deleted')),
    message_id UUID NOT NULL REFERENCES mahfel.chat_messages (id) ON DELETE CASCADE,
    author_actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    body TEXT NULL,
    message_created_at TIMESTAMPTZ NOT NULL,
    message_edited_at TIMESTAMPTZ NULL,
    message_deleted_at TIMESTAMPTZ NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (channel_id, event_seq),
    CHECK ((kind = 'deleted') = (body IS NULL))
);

CREATE OR REPLACE FUNCTION mahfel.append_chat_message_event()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    next_sequence BIGINT;
    event_kind TEXT;
BEGIN
    IF TG_OP = 'INSERT' THEN
        event_kind := 'created';
    ELSIF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
        event_kind := 'deleted';
    ELSIF NEW.body IS DISTINCT FROM OLD.body OR NEW.edited_at IS DISTINCT FROM OLD.edited_at THEN
        event_kind := 'edited';
    ELSE
        RETURN NEW;
    END IF;

    UPDATE mahfel.chat_channels
    SET event_sequence = event_sequence + 1
    WHERE id = NEW.channel_id
    RETURNING event_sequence INTO next_sequence;

    INSERT INTO mahfel.chat_events (
        channel_id, event_seq, id, kind, message_id, author_actor_id,
        body, message_created_at, message_edited_at, message_deleted_at, occurred_at
    ) VALUES (
        NEW.channel_id, next_sequence, gen_random_uuid(), event_kind,
        NEW.id, NEW.author_actor_id,
        CASE WHEN NEW.deleted_at IS NULL THEN NEW.body ELSE NULL END,
        NEW.created_at, NEW.edited_at, NEW.deleted_at, clock_timestamp()
    );
    RETURN NEW;
END;
$$;

CREATE TRIGGER chat_message_event_insert
AFTER INSERT ON mahfel.chat_messages
FOR EACH ROW EXECUTE FUNCTION mahfel.append_chat_message_event();

CREATE TRIGGER chat_message_event_update
AFTER UPDATE OF body, deleted_at ON mahfel.chat_messages
FOR EACH ROW EXECUTE FUNCTION mahfel.append_chat_message_event();
