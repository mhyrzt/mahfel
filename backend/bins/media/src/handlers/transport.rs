use crate::validation::{ConnectTransportValidated, CreateTransportValidated, Validated};
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
    let Validated(CreateTransportValidated {
        room_id,
        participant_id,
        ..
    }) = request.into_inner().try_into()?;
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
    let Validated(ConnectTransportValidated {
        room_id,
        transport_id,
        dtls_parameters,
    }) = request.into_inner().try_into()?;
    media
        .connect_transport(room_id, &transport_id, &dtls_parameters)
        .await?;
    Ok(Response::new(ConnectWebRtcTransportResponse {}))
}
