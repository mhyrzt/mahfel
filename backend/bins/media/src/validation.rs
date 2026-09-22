use mahfel_domain::{ActorId, ProducerSource, RoomId};
use mediasoup::prelude::MediaKind;
use tonic::Status;
use uuid::Uuid;

#[inline]
pub(crate) fn saturating_u64(v: usize) -> u64 {
    u64::try_from(v).unwrap_or(u64::MAX)
}

#[inline]
pub(crate) fn saturating_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

#[inline]
pub(crate) fn saturating_i64(v: usize) -> i64 {
    i64::try_from(v).unwrap_or(0)
}

pub(crate) fn require_field(raw: String, field: &str) -> Result<String, Status> {
    let trimmed = raw.trim().to_owned();
    if trimmed.is_empty() {
        return Err(Status::invalid_argument(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed)
}

pub(crate) fn parse_room_id(raw: &str) -> Result<RoomId, Status> {
    let uuid = Uuid::parse_str(raw).map_err(|_| Status::invalid_argument("invalid room_id"))?;
    Ok(RoomId::from_uuid(uuid))
}

pub(crate) fn parse_user_id(raw: &str) -> Result<ActorId, Status> {
    let uuid =
        Uuid::parse_str(raw).map_err(|_| Status::invalid_argument("invalid participant_id"))?;
    Ok(ActorId::from_uuid(uuid))
}

pub(crate) fn parse_media_kind(raw: &str) -> Result<MediaKind, Status> {
    match raw {
        "audio" => Ok(MediaKind::Audio),
        "video" => Ok(MediaKind::Video),
        _ => Err(Status::invalid_argument("invalid kind")),
    }
}

/// The application-level intent of a producer.
///
/// An empty source takes the kind's obvious meaning, so a caller that predates
/// the field keeps working: audio is a microphone and video is a camera.
pub(crate) fn parse_producer_source(raw: &str, kind: MediaKind) -> Result<ProducerSource, Status> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(match kind {
            MediaKind::Audio => ProducerSource::Microphone,
            MediaKind::Video => ProducerSource::Camera,
        });
    }
    let source: ProducerSource = trimmed
        .try_into()
        .map_err(|_| Status::invalid_argument("invalid source"))?;
    // A source decides which producers a moderation action selects, so a
    // mismatch would let a caller hide an audio track behind a video label.
    if source.is_audio() != (kind == MediaKind::Audio) {
        return Err(Status::invalid_argument("source does not match kind"));
    }
    Ok(source)
}

pub struct Validated<T>(pub T);

pub struct CreateRoomValidated {
    pub room_id: RoomId,
    pub raw_room_id: String,
}
pub struct DeleteRoomValidated {
    pub room_id: RoomId,
}
pub struct CreateTransportValidated {
    pub room_id: RoomId,
    pub participant_id: ActorId,
}
pub struct ConnectTransportValidated {
    pub room_id: RoomId,
    pub transport_id: String,
    pub dtls_parameters: String,
}
pub struct ProduceValidated {
    pub room_id: RoomId,
    pub participant_id: ActorId,
    pub transport_id: String,
    pub kind: MediaKind,
    pub source: ProducerSource,
    pub rtp_parameters: String,
}
pub struct ConsumeValidated {
    pub room_id: RoomId,
    pub participant_id: ActorId,
    pub producer_id: String,
    pub transport_id: String,
    pub rtp_capabilities: String,
}
pub struct PauseValidated {
    pub room_id: RoomId,
    pub consumer_id: String,
}
pub struct ProducerValidated {
    pub room_id: RoomId,
    pub producer_id: String,
}
pub struct ParticipantSourceValidated {
    pub room_id: RoomId,
    pub participant_id: ActorId,
    pub source: ProducerSource,
}
pub struct CloseParticipantValidated {
    pub room_id: RoomId,
    pub participant_id: ActorId,
}

impl TryFrom<mahfel_rtc_protocol::media::CreateRoomRequest> for Validated<CreateRoomValidated> {
    type Error = Status;
    fn try_from(req: mahfel_rtc_protocol::media::CreateRoomRequest) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let room_id = parse_room_id(&raw_room_id)?;
        Ok(Self(CreateRoomValidated {
            room_id,
            raw_room_id,
        }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::DeleteRoomRequest> for Validated<DeleteRoomValidated> {
    type Error = Status;
    fn try_from(req: mahfel_rtc_protocol::media::DeleteRoomRequest) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let room_id = parse_room_id(&raw_room_id)?;
        Ok(Self(DeleteRoomValidated { room_id }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::CreateWebRtcTransportRequest>
    for Validated<CreateTransportValidated>
{
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::CreateWebRtcTransportRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let raw_participant = require_field(req.participant_id, "participant_id")?;
        Ok(Self(CreateTransportValidated {
            room_id: parse_room_id(&raw_room_id)?,
            participant_id: parse_user_id(&raw_participant)?,
        }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::ConnectWebRtcTransportRequest>
    for Validated<ConnectTransportValidated>
{
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::ConnectWebRtcTransportRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let transport_id = require_field(req.transport_id, "transport_id")?;
        let dtls_parameters = require_field(req.dtls_parameters, "dtls_parameters")?;
        Ok(Self(ConnectTransportValidated {
            room_id: parse_room_id(&raw_room_id)?,
            transport_id,
            dtls_parameters,
        }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::ProduceRequest> for Validated<ProduceValidated> {
    type Error = Status;
    fn try_from(req: mahfel_rtc_protocol::media::ProduceRequest) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let raw_participant = require_field(req.participant_id, "participant_id")?;
        let transport_id = require_field(req.transport_id, "transport_id")?;
        let raw_kind = require_field(req.kind, "kind")?;
        let rtp_parameters = require_field(req.rtp_parameters, "rtp_parameters")?;
        let kind = parse_media_kind(&raw_kind)?;
        Ok(Self(ProduceValidated {
            room_id: parse_room_id(&raw_room_id)?,
            participant_id: parse_user_id(&raw_participant)?,
            transport_id,
            kind,
            source: parse_producer_source(&req.source, kind)?,
            rtp_parameters,
        }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::ConsumeRequest> for Validated<ConsumeValidated> {
    type Error = Status;
    fn try_from(req: mahfel_rtc_protocol::media::ConsumeRequest) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let raw_participant = require_field(req.participant_id, "participant_id")?;
        let producer_id = require_field(req.producer_id, "producer_id")?;
        let transport_id = require_field(req.transport_id, "transport_id")?;
        let rtp_capabilities = require_field(req.rtp_capabilities, "rtp_capabilities")?;
        Ok(Self(ConsumeValidated {
            room_id: parse_room_id(&raw_room_id)?,
            participant_id: parse_user_id(&raw_participant)?,
            producer_id,
            transport_id,
            rtp_capabilities,
        }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::PauseConsumerRequest> for Validated<PauseValidated> {
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::PauseConsumerRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let consumer_id = require_field(req.consumer_id, "consumer_id")?;
        Ok(Self(PauseValidated {
            room_id: parse_room_id(&raw_room_id)?,
            consumer_id,
        }))
    }
}
impl TryFrom<mahfel_rtc_protocol::media::ResumeConsumerRequest> for Validated<PauseValidated> {
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::ResumeConsumerRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let consumer_id = require_field(req.consumer_id, "consumer_id")?;
        Ok(Self(PauseValidated {
            room_id: parse_room_id(&raw_room_id)?,
            consumer_id,
        }))
    }
}

impl TryFrom<mahfel_rtc_protocol::media::PauseProducerRequest> for Validated<ProducerValidated> {
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::PauseProducerRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let producer_id = require_field(req.producer_id, "producer_id")?;
        Ok(Self(ProducerValidated {
            room_id: parse_room_id(&raw_room_id)?,
            producer_id,
        }))
    }
}

impl TryFrom<mahfel_rtc_protocol::media::ResumeProducerRequest> for Validated<ProducerValidated> {
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::ResumeProducerRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let producer_id = require_field(req.producer_id, "producer_id")?;
        Ok(Self(ProducerValidated {
            room_id: parse_room_id(&raw_room_id)?,
            producer_id,
        }))
    }
}

impl TryFrom<mahfel_rtc_protocol::media::PauseParticipantSourceRequest>
    for Validated<ParticipantSourceValidated>
{
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::PauseParticipantSourceRequest,
    ) -> Result<Self, Self::Error> {
        participant_source_validated(req.room_id, req.participant_id, req.source)
    }
}

impl TryFrom<mahfel_rtc_protocol::media::ResumeParticipantSourceRequest>
    for Validated<ParticipantSourceValidated>
{
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::ResumeParticipantSourceRequest,
    ) -> Result<Self, Self::Error> {
        participant_source_validated(req.room_id, req.participant_id, req.source)
    }
}

fn participant_source_validated(
    raw_room_id: String,
    raw_participant: String,
    raw_source: String,
) -> Result<Validated<ParticipantSourceValidated>, Status> {
    let raw_room_id = require_field(raw_room_id, "room_id")?;
    let raw_participant = require_field(raw_participant, "participant_id")?;
    let raw_source = require_field(raw_source, "source")?;
    let source: ProducerSource = raw_source
        .trim()
        .try_into()
        .map_err(|_| Status::invalid_argument("invalid source"))?;
    Ok(Validated(ParticipantSourceValidated {
        room_id: parse_room_id(&raw_room_id)?,
        participant_id: parse_user_id(&raw_participant)?,
        source,
    }))
}

impl TryFrom<mahfel_rtc_protocol::media::CloseParticipantSourceRequest>
    for Validated<ParticipantSourceValidated>
{
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::CloseParticipantSourceRequest,
    ) -> Result<Self, Self::Error> {
        participant_source_validated(req.room_id, req.participant_id, req.source)
    }
}

impl TryFrom<mahfel_rtc_protocol::media::CloseParticipantRequest>
    for Validated<CloseParticipantValidated>
{
    type Error = Status;
    fn try_from(
        req: mahfel_rtc_protocol::media::CloseParticipantRequest,
    ) -> Result<Self, Self::Error> {
        let raw_room_id = require_field(req.room_id, "room_id")?;
        let raw_participant = require_field(req.participant_id, "participant_id")?;
        Ok(Self(CloseParticipantValidated {
            room_id: parse_room_id(&raw_room_id)?,
            participant_id: parse_user_id(&raw_participant)?,
        }))
    }
}
