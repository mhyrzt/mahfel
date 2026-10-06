use crate::ports::{BoxFuture, Principal, StoreError};
use crate::{ApplicationError, ErrorKind};
use mahfel_domain::{ActivityDefinition, ActorId, ParticipantId, SessionId};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum InteractionError {
    #[error("{0}")]
    Invalid(#[from] mahfel_domain::ActivityError),
    #[error("session interactions unavailable")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("{0}")]
    Conflict(&'static str),
    #[error("{0}")]
    Store(#[from] StoreError),
}
impl ApplicationError for InteractionError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Invalid(_) => ErrorKind::Unprocessable,
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Conflict(_) => ErrorKind::Conflict,
            Self::Store(StoreError::NotFound(_) | StoreError::MissingReference(_)) => {
                ErrorKind::NotFound
            }
            Self::Store(StoreError::Conflict(_)) => ErrorKind::Conflict,
            Self::Store(StoreError::Unexpected(_)) => ErrorKind::Unavailable,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActivityRecord {
    pub id: Uuid,
    pub definition: ActivityDefinition,
    pub status: ActivityStatus,
    pub votes: Vec<ActivityVote>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityStatus {
    Draft,
    Open,
    Closed,
}
#[derive(Clone, Debug)]
pub struct ActivityVote {
    pub actor_id: ActorId,
    pub name: String,
    pub selected: Vec<u16>,
    pub version: i64,
}
#[derive(Clone, Debug)]
pub struct RaisedHand {
    pub participant_id: ParticipantId,
    pub actor_id: ActorId,
    pub name: String,
    pub order: i64,
}
#[derive(Clone, Debug)]
pub struct InteractionSnapshot {
    pub version: i64,
    pub live: bool,
    pub can_manage: bool,
    pub can_moderate: bool,
    pub can_respond: bool,
    pub activities: Vec<ActivityRecord>,
    pub hands: Vec<RaisedHand>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum InteractionCommand {
    SaveDraft {
        id: Uuid,
        definition: ActivityDefinition,
    },
    Launch {
        id: Uuid,
    },
    Close {
        id: Uuid,
    },
    Vote {
        id: Uuid,
        selected: Vec<u16>,
        expected_version: i64,
    },
    Retract {
        id: Uuid,
        expected_version: i64,
    },
    SetHand {
        raised: bool,
        expected_version: i64,
    },
    LowerHand {
        participant_id: ParticipantId,
        expected_order: i64,
    },
    ClearHands {
        expected_version: i64,
    },
}

pub trait InteractionStore: Send + Sync {
    fn can_respond<'a>(
        &'a self,
        session_id: SessionId,
        principal: &'a Principal,
    ) -> BoxFuture<'a, Result<bool, InteractionError>> {
        Box::pin(async move { Ok(self.snapshot(session_id, principal).await?.can_respond) })
    }
    fn snapshot<'a>(
        &'a self,
        session_id: SessionId,
        principal: &'a Principal,
    ) -> BoxFuture<'a, Result<InteractionSnapshot, InteractionError>>;
    fn execute<'a>(
        &'a self,
        session_id: SessionId,
        principal: &'a Principal,
        request_id: Uuid,
        command: InteractionCommand,
    ) -> BoxFuture<'a, Result<(), InteractionError>>;
    fn version(&self, session_id: SessionId) -> BoxFuture<'_, Result<i64, InteractionError>>;
    fn clear_departed(
        &self,
        session_id: SessionId,
        participant_id: ParticipantId,
    ) -> BoxFuture<'_, Result<(), InteractionError>>;
    fn raised_hands(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<RaisedHand>, InteractionError>>;
}
