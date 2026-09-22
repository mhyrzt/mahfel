-- Room-scoped admission: a room invite link asks to join the room itself,
-- which may have no live session at all.
--
-- Until now every pending request belonged to a session, because the only way
-- into the lobby was attempting a join. A room invite link breaks that: it is
-- durable onboarding, so it must be approvable with no session running, and
-- approval grants room membership rather than session participation (ADR 0021).
ALTER TABLE mahfel.admission_requests
ADD COLUMN room_id UUID NULL REFERENCES mahfel.rooms (id) ON DELETE CASCADE;

-- Backfill before the scope rule is enforced: every existing request was a
-- session request, so its room comes from its session.
UPDATE mahfel.admission_requests AS request
SET room_id = owning_session.room_id
FROM mahfel.sessions AS owning_session
WHERE request.session_id = owning_session.id;

-- A request must name a room, because that is what authorization resolves
-- against even for a session request.
ALTER TABLE mahfel.admission_requests
ALTER COLUMN room_id SET NOT NULL;

ALTER TABLE mahfel.admission_requests
ALTER COLUMN session_id DROP NOT NULL;

-- The room is always present, because authorization resolves against it for
-- both scopes. A session id distinguishes the scopes: present means the request
-- is for that one session, absent means it is for the room itself. This is the
-- invariant the domain's AdmissionTarget enum encodes in memory.

-- The old partial indexes keyed only on the session, which is now nullable.
DROP INDEX mahfel.admission_requests_one_pending_idx;
DROP INDEX mahfel.admission_requests_session_pending_idx;

-- At most one outstanding request per principal and target, so a repeated
-- redemption of the same link cannot queue twice.
CREATE UNIQUE INDEX admission_requests_one_pending_idx
ON mahfel.admission_requests (session_id, actor_id)
WHERE status = 'pending' AND session_id IS NOT NULL;

CREATE UNIQUE INDEX admission_requests_one_pending_room_idx
ON mahfel.admission_requests (room_id, actor_id)
WHERE status = 'pending' AND session_id IS NULL;

-- A manager reads one session's queue, oldest first.
CREATE INDEX admission_requests_session_pending_idx
ON mahfel.admission_requests (session_id, requested_at)
WHERE status = 'pending' AND session_id IS NOT NULL;

-- A manager reads one room's queue, oldest first.
CREATE INDEX admission_requests_room_pending_idx
ON mahfel.admission_requests (room_id, requested_at)
WHERE status = 'pending' AND session_id IS NULL;
