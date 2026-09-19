use crate::http::error::api_error_from;
use mahfel_application::rooms::{
    AddMemberError, ArchiveRoomError, CreateRoomError, GetRoomError, JoinRoomError, ListRoomsError,
    RemoveMemberError, UpdateRoomError,
};

api_error_from!(
    CreateRoomError,
    ListRoomsError,
    GetRoomError,
    JoinRoomError,
    AddMemberError,
    RemoveMemberError,
    UpdateRoomError,
    ArchiveRoomError,
);
