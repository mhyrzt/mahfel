use super::BoxFuture;
use mahfel_domain::{ActorId, ProducerSource, RoomId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaGatewayError {
    #[error("media gateway failure: {0}")]
    Transport(String),
    #[error("media resource not found: {0}")]
    NotFound(String),
}

impl MediaGatewayError {
    pub fn transport<E: std::fmt::Display>(err: E) -> Self {
        Self::Transport(err.to_string())
    }
}

/// Media-plane control. Methods return [`BoxFuture`] so the port is
/// dyn-compatible.
pub trait MediaGateway: Send + Sync {
    fn create_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>>;

    fn close_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>>;

    /// Pause one producer without destroying it.
    ///
    /// Pausing is what lets an enforced mute map onto media state one-to-one:
    /// resuming needs no renegotiation, and a caller's durable mute record
    /// never points at a producer that no longer exists.
    fn pause_producer(
        &self,
        room: RoomId,
        producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>>;

    fn resume_producer(
        &self,
        room: RoomId,
        producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>>;

    /// Close every producer a participant owns.
    ///
    /// Removal is terminal, so closing is correct rather than pausing. A
    /// participant with nothing to close succeeds, because whether the
    /// participant exists is the caller's check against the authoritative
    /// store; the media plane holds no participant registry and must not.
    fn close_participant(
        &self,
        room: RoomId,
        participant_id: ActorId,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>>;

    /// Pause every producer a participant owns for one source.
    ///
    /// Source targeting is what makes muting a microphone distinct from
    /// stopping a screen share: both are video or audio to the SFU. Returns how
    /// many were affected, which is zero when the participant has published
    /// nothing of that source.
    fn pause_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>>;

    fn resume_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>>;

    /// Close every producer a participant owns for one source.
    ///
    /// Closing rather than pausing is what a capability withdrawal needs: a
    /// paused producer would have nobody to resume it. Returns how many were
    /// closed, which is zero when the participant has published nothing of that
    /// source.
    fn close_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>>;
}
