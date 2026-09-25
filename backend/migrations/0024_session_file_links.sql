CREATE TABLE mahfel.session_file_links (
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES mahfel.files (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, file_id)
);

CREATE INDEX session_file_links_file_idx ON mahfel.session_file_links (file_id);
