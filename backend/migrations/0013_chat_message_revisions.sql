ALTER TABLE mahfel.chat_messages
ADD COLUMN edited_at TIMESTAMPTZ NULL
CHECK (edited_at IS NULL OR edited_at >= created_at);

CREATE TABLE mahfel.chat_message_revisions (
    id UUID PRIMARY KEY,
    message_id UUID NOT NULL
    REFERENCES mahfel.chat_messages (id) ON DELETE CASCADE,
    editor_actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    prior_body TEXT NOT NULL CHECK (length(btrim(prior_body)) > 0),
    revision_kind TEXT NOT NULL CHECK (revision_kind IN ('edited', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX chat_message_revisions_timeline_idx
ON mahfel.chat_message_revisions (message_id, created_at, id);
