use crate::{ActorId, SessionId, ids::entity_id, status::status_conversion};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(RecordingId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatus {
    Starting,
    Recording,
    Paused,
    Stopping,
    Finalizing,
    Ready,
    Failed,
}

status_conversion!(RecordingStatus, "recording", {
    Starting => "starting", Recording => "recording", Paused => "paused",
    Stopping => "stopping", Finalizing => "finalizing", Ready => "ready", Failed => "failed",
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingAction {
    Start,
    Pause,
    Resume,
    Stop,
}

status_conversion!(RecordingAction, "recording action", {
    Start => "start", Pause => "pause", Resume => "resume", Stop => "stop",
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recording {
    pub id: RecordingId,
    pub session_id: SessionId,
    pub status: RecordingStatus,
    pub revision: i64,
    pub capture_generation: i32,
    pub initiated_by: Option<ActorId>,
    pub started_at: OffsetDateTime,
    pub stopped_at: Option<OffsetDateTime>,
    pub warning: Option<String>,
    pub partial: bool,
    pub manifest_key: Option<String>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("recording command conflicts with current state")]
pub struct RecordingTransitionError;

impl RecordingStatus {
    #[must_use]
    pub const fn unfinished(self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Recording | Self::Paused | Self::Stopping
        )
    }

    pub fn transition(self, action: RecordingAction) -> Result<Self, RecordingTransitionError> {
        match (self, action) {
            (Self::Starting | Self::Recording, RecordingAction::Pause) => Ok(Self::Paused),
            (Self::Paused, RecordingAction::Resume) => Ok(Self::Starting),
            (Self::Starting | Self::Recording | Self::Paused, RecordingAction::Stop) => {
                Ok(Self::Stopping)
            }
            (Self::Paused, RecordingAction::Pause) | (Self::Stopping, RecordingAction::Stop) => {
                Ok(self)
            }
            _ => Err(RecordingTransitionError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_resume_and_stop_preserve_one_recording_lifecycle() {
        let recording = RecordingStatus::Recording;
        assert_eq!(
            recording.transition(RecordingAction::Pause),
            Ok(RecordingStatus::Paused)
        );
        assert_eq!(
            RecordingStatus::Paused.transition(RecordingAction::Resume),
            Ok(RecordingStatus::Starting)
        );
        for status in [
            RecordingStatus::Starting,
            recording,
            RecordingStatus::Paused,
        ] {
            assert_eq!(
                status.transition(RecordingAction::Stop),
                Ok(RecordingStatus::Stopping)
            );
        }
        assert_eq!(
            RecordingStatus::Stopping.transition(RecordingAction::Stop),
            Ok(RecordingStatus::Stopping)
        );
    }

    #[test]
    fn finalized_recordings_cannot_resume_or_start_in_place() {
        for status in [
            RecordingStatus::Finalizing,
            RecordingStatus::Ready,
            RecordingStatus::Failed,
        ] {
            assert!(!status.unfinished());
            for action in [
                RecordingAction::Start,
                RecordingAction::Pause,
                RecordingAction::Resume,
                RecordingAction::Stop,
            ] {
                assert!(status.transition(action).is_err());
            }
        }
        assert!(
            RecordingStatus::Recording
                .transition(RecordingAction::Start)
                .is_err()
        );
    }
}
