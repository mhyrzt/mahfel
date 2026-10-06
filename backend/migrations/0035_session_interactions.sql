ALTER TABLE mahfel.sessions ADD COLUMN interaction_version BIGINT NOT NULL DEFAULT 0;

CREATE TABLE mahfel.session_activities (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id),
    created_by UUID NOT NULL REFERENCES mahfel.actors (id),
    definition JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'open', 'closed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at TIMESTAMPTZ,
    UNIQUE (id, session_id)
);
CREATE UNIQUE INDEX session_activities_one_open
ON mahfel.session_activities (session_id) WHERE status = 'open';
CREATE INDEX session_activities_session ON mahfel.session_activities (session_id, created_at);

CREATE TABLE mahfel.activity_votes (
    activity_id UUID NOT NULL REFERENCES mahfel.session_activities (id),
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    vote_version BIGINT NOT NULL DEFAULT 1,
    selected SMALLINT [] NOT NULL CHECK (cardinality(selected) BETWEEN 0 AND 10),
    PRIMARY KEY (activity_id, actor_id)
);

CREATE TABLE mahfel.raised_hands (
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id),
    participant_id UUID NOT NULL REFERENCES mahfel.session_participants (id),
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    queue_order BIGINT GENERATED ALWAYS AS IDENTITY,
    PRIMARY KEY (session_id, actor_id),
    UNIQUE (participant_id),
    UNIQUE (queue_order)
);

CREATE FUNCTION mahfel.clear_session_interactions() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.status IN ('ended', 'cancelled') AND OLD.status <> NEW.status THEN
        UPDATE mahfel.session_activities SET status = 'closed', closed_at = now()
        WHERE session_id = NEW.id AND status = 'open';
        DELETE FROM mahfel.raised_hands WHERE session_id = NEW.id;
        NEW.interaction_version := OLD.interaction_version + 1;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER clear_session_interactions BEFORE UPDATE OF status ON mahfel.sessions
FOR EACH ROW EXECUTE FUNCTION mahfel.clear_session_interactions();

CREATE FUNCTION mahfel.clear_participant_hand() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.status <> 'active' AND OLD.status = 'active' THEN
        DELETE FROM mahfel.raised_hands WHERE participant_id = NEW.id;
        IF FOUND THEN
            UPDATE mahfel.sessions SET interaction_version = interaction_version + 1
            WHERE id = NEW.session_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER clear_participant_hand AFTER UPDATE OF status ON mahfel.session_participants
FOR EACH ROW EXECUTE FUNCTION mahfel.clear_participant_hand();

CREATE TABLE mahfel.interaction_commands (
    session_id UUID NOT NULL REFERENCES mahfel.sessions (id),
    actor_id UUID NOT NULL REFERENCES mahfel.actors (id),
    id UUID NOT NULL,
    command JSONB NOT NULL,
    PRIMARY KEY (session_id, actor_id, id)
);
