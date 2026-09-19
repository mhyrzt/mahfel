ALTER TABLE mahfel.rooms
ADD COLUMN visibility TEXT NOT NULL DEFAULT 'invite_only'
CHECK (visibility IN ('public', 'invite_only'));

CREATE TABLE mahfel.room_memberships (
    room_id UUID NOT NULL REFERENCES mahfel.rooms (id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES mahfel.users (id) ON DELETE CASCADE,
    member_role TEXT NOT NULL CHECK (member_role IN ('manager', 'member')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (room_id, user_id),
    CHECK (updated_at >= created_at)
);

CREATE INDEX room_memberships_user_id_idx
ON mahfel.room_memberships (user_id);

CREATE INDEX room_memberships_room_id_role_idx
ON mahfel.room_memberships (room_id, member_role);
