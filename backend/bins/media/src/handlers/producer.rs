use crate::validation::{parse_media_kind, parse_room_id, parse_user_id, require_field};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{ProduceRequest, ProduceResponse};
use tonic::{Request, Response, Status};

pub(crate) async fn produce(
    media: &MediaRuntime,
    request: Request<ProduceRequest>,
) -> Result<Response<ProduceResponse>, Status> {
    let inner = request.into_inner();
    let raw_room = require_field(inner.room_id, "room_id")?;
    let raw_participant = require_field(inner.participant_id, "participant_id")?;
    let transport_id = require_field(inner.transport_id, "transport_id")?;
    let raw_kind = require_field(inner.kind, "kind")?;
    let rtp_parameters = require_field(inner.rtp_parameters, "rtp_parameters")?;
    let room_id = parse_room_id(&raw_room)?;
    let participant_id = parse_user_id(&raw_participant)?;
    let kind = parse_media_kind(&raw_kind)?;
    let producer_id = media
        .create_producer(
            room_id,
            &transport_id,
            participant_id,
            kind,
            &rtp_parameters,
        )
        .await?;
    Ok(Response::new(ProduceResponse { producer_id }))
}
