ALTER TABLE mahfel.chat_messages
ADD COLUMN parent_id UUID NULL REFERENCES mahfel.chat_messages (id) ON DELETE RESTRICT,
ADD CONSTRAINT chat_messages_not_self_parent CHECK (parent_id IS NULL OR parent_id <> id);

CREATE INDEX chat_messages_thread_timeline_idx
ON mahfel.chat_messages (parent_id, created_at DESC, id DESC)
WHERE parent_id IS NOT NULL;

CREATE OR REPLACE FUNCTION mahfel.check_chat_parent()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.parent_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM mahfel.chat_messages parent
        WHERE parent.id = NEW.parent_id
          AND parent.channel_id = NEW.channel_id
          AND parent.parent_id IS NULL
          AND parent.deleted_at IS NULL
    ) THEN
        RAISE EXCEPTION 'invalid chat parent' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER chat_message_parent_check
BEFORE INSERT OR UPDATE OF parent_id, channel_id ON mahfel.chat_messages
FOR EACH ROW EXECUTE FUNCTION mahfel.check_chat_parent();

CREATE TABLE mahfel.chat_mentions (
    message_id UUID NOT NULL REFERENCES mahfel.chat_messages (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    PRIMARY KEY (message_id, actor_id)
);

CREATE TABLE mahfel.chat_reactions (
    message_id UUID NOT NULL REFERENCES mahfel.chat_messages (id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    emoji TEXT NOT NULL CHECK (emoji IN ('👍', '👎', '❤️', '😂', '😮', '😢')),
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (message_id, actor_id)
);

CREATE INDEX chat_reactions_message_emoji_idx
ON mahfel.chat_reactions (message_id, emoji);

ALTER TABLE mahfel.chat_events
ADD COLUMN parent_id UUID NULL,
ADD COLUMN reaction_actor_id UUID NULL REFERENCES mahfel.actors (id),
ADD COLUMN reaction_emoji TEXT NULL;

ALTER TABLE mahfel.chat_events
DROP CONSTRAINT chat_events_kind_check;

ALTER TABLE mahfel.chat_events
ADD CONSTRAINT chat_events_kind_check
CHECK (kind IN ('created', 'edited', 'deleted', 'reaction_set', 'reaction_removed'));

ALTER TABLE mahfel.chat_events
DROP CONSTRAINT chat_events_check;

ALTER TABLE mahfel.chat_events
ADD CONSTRAINT chat_events_body_check
CHECK (kind NOT IN ('created', 'edited', 'deleted') OR ((kind = 'deleted') = (body IS NULL)));

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
        body, message_created_at, message_edited_at, message_deleted_at,
        occurred_at, parent_id
    ) VALUES (
        NEW.channel_id, next_sequence, gen_random_uuid(), event_kind,
        NEW.id, NEW.author_actor_id,
        CASE WHEN NEW.deleted_at IS NULL THEN NEW.body ELSE NULL END,
        NEW.created_at, NEW.edited_at, NEW.deleted_at, clock_timestamp(), NEW.parent_id
    );
    RETURN NEW;
END;
$$;
