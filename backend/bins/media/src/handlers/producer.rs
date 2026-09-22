use crate::validation::{
    CloseParticipantValidated, ProduceValidated, ProducerValidated, Validated,
};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    CloseParticipantRequest, CloseParticipantResponse, PauseProducerRequest, PauseProducerResponse,
    ProduceRequest, ProduceResponse, ResumeProducerRequest, ResumeProducerResponse,
};
use tonic::{Request, Response, Status};

pub(crate) async fn produce(
    media: &MediaRuntime,
    request: Request<ProduceRequest>,
) -> Result<Response<ProduceResponse>, Status> {
    let Validated(ProduceValidated {
        room_id,
        participant_id,
        transport_id,
        kind,
        rtp_parameters,
        ..
    }) = request.into_inner().try_into()?;
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

pub(crate) async fn pause_producer(
    media: &MediaRuntime,
    request: Request<PauseProducerRequest>,
) -> Result<Response<PauseProducerResponse>, Status> {
    let Validated(ProducerValidated {
        room_id,
        producer_id,
        ..
    }) = request.into_inner().try_into()?;
    media.pause_producer(room_id, &producer_id).await?;
    Ok(Response::new(PauseProducerResponse {}))
}

pub(crate) async fn resume_producer(
    media: &MediaRuntime,
    request: Request<ResumeProducerRequest>,
) -> Result<Response<ResumeProducerResponse>, Status> {
    let Validated(ProducerValidated {
        room_id,
        producer_id,
        ..
    }) = request.into_inner().try_into()?;
    media.resume_producer(room_id, &producer_id).await?;
    Ok(Response::new(ResumeProducerResponse {}))
}

/// Close every producer one participant owns.
///
/// Succeeds with a zero count when the participant published nothing, so
/// removing a silent participant is not an error.
pub(crate) async fn close_participant(
    media: &MediaRuntime,
    request: Request<CloseParticipantRequest>,
) -> Result<Response<CloseParticipantResponse>, Status> {
    let Validated(CloseParticipantValidated {
        room_id,
        participant_id,
        ..
    }) = request.into_inner().try_into()?;
    media.close_participant(room_id, participant_id).await?;
    Ok(Response::new(CloseParticipantResponse {}))
}
