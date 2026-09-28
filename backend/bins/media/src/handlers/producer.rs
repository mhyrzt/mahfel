use crate::validation::{
    CloseParticipantValidated, ParticipantSourceValidated, ProduceValidated, ProducerValidated,
    Validated, parse_room_id,
};
use mahfel_rtc::{MediaKind, MediaRuntime};
use mahfel_rtc_protocol::media::{
    CloseParticipantRequest, CloseParticipantResponse, CloseParticipantSourceRequest,
    CloseParticipantSourceResponse, ListProducersRequest, ListProducersResponse,
    PauseParticipantSourceRequest, PauseParticipantSourceResponse, PauseProducerRequest,
    PauseProducerResponse, ProduceRequest, ProduceResponse, ProducerInfo,
    ResumeParticipantSourceRequest, ResumeParticipantSourceResponse, ResumeProducerRequest,
    ResumeProducerResponse,
};
use tonic::{Request, Response, Status};

pub(crate) async fn list_producers(
    media: &MediaRuntime,
    request: Request<ListProducersRequest>,
) -> Result<Response<ListProducersResponse>, Status> {
    let room_id = parse_room_id(&request.into_inner().room_id)?;
    let producers = media
        .list_producers(room_id)
        .await
        .into_iter()
        .map(|entry| ProducerInfo {
            producer_id: entry.producer_id,
            actor_id: entry.actor_id.to_string(),
            kind: match entry.kind {
                MediaKind::Audio => "audio",
                MediaKind::Video => "video",
            }
            .to_owned(),
            source: entry.source.as_str().to_owned(),
        })
        .collect();
    Ok(Response::new(ListProducersResponse { producers }))
}

pub(crate) async fn produce(
    media: &MediaRuntime,
    request: Request<ProduceRequest>,
) -> Result<Response<ProduceResponse>, Status> {
    let Validated(ProduceValidated {
        room_id,
        participant_id,
        transport_id,
        kind,
        source,
        rtp_parameters,
        ..
    }) = request.into_inner().try_into()?;
    let producer_id = media
        .create_producer(
            room_id,
            &transport_id,
            participant_id,
            kind,
            source,
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
    let closed = media.close_participant(room_id, participant_id).await?;
    Ok(Response::new(CloseParticipantResponse {
        closed: u32::try_from(closed).unwrap_or(u32::MAX),
    }))
}

/// Close every producer one participant owns for one source.
pub(crate) async fn close_participant_source(
    media: &MediaRuntime,
    request: Request<CloseParticipantSourceRequest>,
) -> Result<Response<CloseParticipantSourceResponse>, Status> {
    let Validated(ParticipantSourceValidated {
        room_id,
        participant_id,
        source,
    }) = request.into_inner().try_into()?;
    let closed = media
        .close_participant_source(room_id, participant_id, source)
        .await?;
    Ok(Response::new(CloseParticipantSourceResponse {
        closed: u32::try_from(closed).unwrap_or(u32::MAX),
    }))
}

/// Pause every producer one participant owns for one source.
pub(crate) async fn pause_participant_source(
    media: &MediaRuntime,
    request: Request<PauseParticipantSourceRequest>,
) -> Result<Response<PauseParticipantSourceResponse>, Status> {
    let Validated(ParticipantSourceValidated {
        room_id,
        participant_id,
        source,
    }) = request.into_inner().try_into()?;
    let affected = media
        .pause_participant_source(room_id, participant_id, source)
        .await?;
    Ok(Response::new(PauseParticipantSourceResponse {
        affected: u32::try_from(affected).unwrap_or(u32::MAX),
    }))
}

pub(crate) async fn resume_participant_source(
    media: &MediaRuntime,
    request: Request<ResumeParticipantSourceRequest>,
) -> Result<Response<ResumeParticipantSourceResponse>, Status> {
    let Validated(ParticipantSourceValidated {
        room_id,
        participant_id,
        source,
    }) = request.into_inner().try_into()?;
    let affected = media
        .resume_participant_source(room_id, participant_id, source)
        .await?;
    Ok(Response::new(ResumeParticipantSourceResponse {
        affected: u32::try_from(affected).unwrap_or(u32::MAX),
    }))
}
