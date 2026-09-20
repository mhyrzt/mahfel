use crate::validation::{parse_room_id, require_field};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    CreateRoomRequest, CreateRoomResponse, DeleteRoomRequest, DeleteRoomResponse,
};
use tonic::{Request, Response, Status};

pub(crate) async fn create_room(
    media: &MediaRuntime,
    request: Request<CreateRoomRequest>,
) -> Result<Response<CreateRoomResponse>, Status> {
    let raw = require_field(request.into_inner().room_id, "room_id")?;
    let room_id = parse_room_id(&raw)?;
    media.create_room(room_id).await?;
    Ok(Response::new(CreateRoomResponse { room_id: raw }))
}

pub(crate) async fn delete_room(
    media: &MediaRuntime,
    request: Request<DeleteRoomRequest>,
) -> Result<Response<DeleteRoomResponse>, Status> {
    let raw = require_field(request.into_inner().room_id, "room_id")?;
    let room_id = parse_room_id(&raw)?;
    let _ = media.delete_room(room_id).await;
    Ok(Response::new(DeleteRoomResponse {}))
}
