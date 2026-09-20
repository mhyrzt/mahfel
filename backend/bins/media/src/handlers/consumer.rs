use crate::validation::{parse_room_id, parse_user_id, require_field};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    ConsumeRequest, ConsumeResponse, PauseConsumerRequest, PauseConsumerResponse,
    ResumeConsumerRequest, ResumeConsumerResponse,
};
use tonic::{Request, Response, Status};

pub(crate) async fn consume(
    media: &MediaRuntime,
    request: Request<ConsumeRequest>,
) -> Result<Response<ConsumeResponse>, Status> {
    let inner = request.into_inner();
    let raw_room = require_field(inner.room_id, "room_id")?;
    let raw_participant = require_field(inner.participant_id, "participant_id")?;
    let producer_id = require_field(inner.producer_id, "producer_id")?;
    let transport_id = require_field(inner.transport_id, "transport_id")?;
    let rtp_capabilities = require_field(inner.rtp_capabilities, "rtp_capabilities")?;
    let room_id = parse_room_id(&raw_room)?;
    let participant_id = parse_user_id(&raw_participant)?;
    let (consumer_id, rtp_parameters, paused) = media
        .create_consumer(
            room_id,
            &transport_id,
            participant_id,
            &producer_id,
            &rtp_capabilities,
        )
        .await?;
    Ok(Response::new(ConsumeResponse {
        consumer_id,
        rtp_parameters,
        paused,
    }))
}

pub(crate) async fn pause_consumer(
    media: &MediaRuntime,
    request: Request<PauseConsumerRequest>,
) -> Result<Response<PauseConsumerResponse>, Status> {
    let inner = request.into_inner();
    let raw_room = require_field(inner.room_id, "room_id")?;
    let consumer_id = require_field(inner.consumer_id, "consumer_id")?;
    let room_id = parse_room_id(&raw_room)?;
    media.pause_consumer(room_id, &consumer_id).await?;
    Ok(Response::new(PauseConsumerResponse {}))
}

pub(crate) async fn resume_consumer(
    media: &MediaRuntime,
    request: Request<ResumeConsumerRequest>,
) -> Result<Response<ResumeConsumerResponse>, Status> {
    let inner = request.into_inner();
    let raw_room = require_field(inner.room_id, "room_id")?;
    let consumer_id = require_field(inner.consumer_id, "consumer_id")?;
    let room_id = parse_room_id(&raw_room)?;
    media.resume_consumer(room_id, &consumer_id).await?;
    Ok(Response::new(ResumeConsumerResponse {}))
}
