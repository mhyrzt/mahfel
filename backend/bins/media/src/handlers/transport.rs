use crate::validation::{parse_room_id, parse_user_id, require_field};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    ConnectWebRtcTransportRequest, ConnectWebRtcTransportResponse, CreateWebRtcTransportRequest,
    CreateWebRtcTransportResponse,
};
use tonic::{Request, Response, Status};

pub(crate) async fn create_web_rtc_transport(
    media: &MediaRuntime,
    request: Request<CreateWebRtcTransportRequest>,
) -> Result<Response<CreateWebRtcTransportResponse>, Status> {
    let inner = request.into_inner();
    let raw_room = require_field(inner.room_id, "room_id")?;
    let raw_participant = require_field(inner.participant_id, "participant_id")?;
    let room_id = parse_room_id(&raw_room)?;
    let participant_id = parse_user_id(&raw_participant)?;
    let (transport_id, ice_parameters, ice_candidates, dtls_parameters) =
        media.create_transport(room_id, participant_id).await?;
    Ok(Response::new(CreateWebRtcTransportResponse {
        transport_id,
        ice_parameters,
        ice_candidates,
        dtls_parameters,
    }))
}

pub(crate) async fn connect_web_rtc_transport(
    media: &MediaRuntime,
    request: Request<ConnectWebRtcTransportRequest>,
) -> Result<Response<ConnectWebRtcTransportResponse>, Status> {
    let inner = request.into_inner();
    let raw_room = require_field(inner.room_id, "room_id")?;
    let transport_id = require_field(inner.transport_id, "transport_id")?;
    let dtls_parameters = require_field(inner.dtls_parameters, "dtls_parameters")?;
    let room_id = parse_room_id(&raw_room)?;
    media
        .connect_transport(room_id, &transport_id, &dtls_parameters)
        .await?;
    Ok(Response::new(ConnectWebRtcTransportResponse {}))
}
