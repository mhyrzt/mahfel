use crate::validation::{ConsumeValidated, PauseValidated, Validated};
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
    let Validated(ConsumeValidated {
        room_id,
        participant_id,
        producer_id,
        transport_id,
        rtp_capabilities,
        ..
    }) = request.into_inner().try_into()?;
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
    let Validated(PauseValidated {
        room_id,
        consumer_id,
        ..
    }) = request.into_inner().try_into()?;
    media.pause_consumer(room_id, &consumer_id).await?;
    Ok(Response::new(PauseConsumerResponse {}))
}

pub(crate) async fn resume_consumer(
    media: &MediaRuntime,
    request: Request<ResumeConsumerRequest>,
) -> Result<Response<ResumeConsumerResponse>, Status> {
    let Validated(PauseValidated {
        room_id,
        consumer_id,
        ..
    }) = request.into_inner().try_into()?;
    media.resume_consumer(room_id, &consumer_id).await?;
    Ok(Response::new(ResumeConsumerResponse {}))
}
