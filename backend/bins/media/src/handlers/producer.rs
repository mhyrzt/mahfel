use crate::validation::{ProduceValidated, Validated};
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{ProduceRequest, ProduceResponse};
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
