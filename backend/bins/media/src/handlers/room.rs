use crate::validation::{
    CreateRoomValidated, DeleteRoomValidated, Validated, parse_room_id, require_field,
};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    ClearRoomMediaRequest, ClearRoomMediaResponse, CreateRoomRequest, CreateRoomResponse,
    DeleteRoomRequest, DeleteRoomResponse,
};
use tonic::{Request, Response, Status};

pub(crate) async fn create_room(
    media: &MediaRuntime,
    request: Request<CreateRoomRequest>,
) -> Result<Response<CreateRoomResponse>, Status> {
    let Validated(CreateRoomValidated {
        room_id,
        raw_room_id,
    }) = request.into_inner().try_into()?;
    media.create_room(room_id).await?;
    Ok(Response::new(CreateRoomResponse {
        room_id: raw_room_id,
    }))
}

pub(crate) async fn clear_room_media(
    media: &MediaRuntime,
    request: Request<ClearRoomMediaRequest>,
) -> Result<Response<ClearRoomMediaResponse>, Status> {
    let room_id = parse_room_id(&require_field(request.into_inner().room_id, "room_id")?)?;
    media.clear_room_media(room_id).await;
    Ok(Response::new(ClearRoomMediaResponse {}))
}

pub(crate) async fn delete_room(
    media: &MediaRuntime,
    request: Request<DeleteRoomRequest>,
) -> Result<Response<DeleteRoomResponse>, Status> {
    let Validated(DeleteRoomValidated { room_id, .. }) = request.into_inner().try_into()?;
    let _ = media.delete_room(room_id).await;
    Ok(Response::new(DeleteRoomResponse {}))
}
