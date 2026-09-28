ALTER TABLE mahfel.chat_messages
ADD COLUMN session_id UUID NULL
REFERENCES mahfel.sessions (id) ON DELETE CASCADE;

CREATE INDEX chat_messages_session_timeline_idx
ON mahfel.chat_messages (channel_id, session_id, created_at DESC, id DESC);
