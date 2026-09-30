//! Durable Mahfel business concepts: identifiers, value objects, aggregate
//! state machines, and their invariants.
//!
//! This crate is independent of infrastructure. Items are re-exported at the
//! crate root (`mahfel_domain::User`, `mahfel_domain::Room`, …) and organized
//! into modules by aggregate.

mod actor;
mod admission;
mod block;
mod chat;
mod collaboration;
mod file;
mod identity;
mod ids;
mod invite;
mod media;
mod participant;
mod participation;
mod recording;
mod recording_export;
mod room;
mod session;
mod status;
mod user;
mod validation;

pub use actor::{
    ActorId, ActorKind, ServicePrincipal, ServicePrincipalError, ServicePrincipalStatus,
};
pub use admission::{
    AdmissionError, AdmissionOrigin, AdmissionPolicy, AdmissionRequest, AdmissionRequestId,
    AdmissionRequestStatus, AdmissionTarget, MAX_PENDING_PER_SESSION,
};
pub use block::{Block, BlockError, BlockId, BlockScope};
pub use chat::{
    CHAT_AUTHOR_MUTATION_WINDOW, CHAT_REACTION_EMOJIS, ChatChannel, ChatChannelId, ChatEvent,
    ChatEventKind, ChatMessage, ChatMessageCursor, ChatMessageDraft, ChatMessageError,
    ChatMessageId, ChatMessagePage,
};
pub use collaboration::{
    CollaborationDocument, CollaborationDocumentId, CollaborationEvent, CollaborationEventKind,
    CollaborationKind,
};
pub use file::{
    FileContext, FileError, FileId, FileObject, FileStatus, MAX_FILE_BYTES, MAX_OWNER_FILE_BYTES,
};
pub use identity::{Identity, IdentityId};
pub use ids::NodeId;
pub use invite::{
    DEFAULT_EXPIRY_DAYS, InviteCode, InviteLink, InviteLinkError, InviteLinkId, InviteLinkStatus,
    InviteTarget, InviteTargetKind, MAX_EXPIRY_DAYS,
};
pub use media::{MediaKind, ProducerSource};
pub use participant::{MuteKind, Participant, ParticipantError, ParticipantId, ParticipantStatus};
pub use participation::{
    CorrectionReason, IntervalError, IntervalId, ParticipationInterval, intervals_overlap,
    total_connected_seconds,
};
pub use recording::{
    Recording, RecordingAction, RecordingId, RecordingStatus, RecordingTransitionError,
};
pub use room::{
    MembershipError, RoleCapability, Room, RoomError, RoomId, RoomMember, RoomRole, RoomStatus,
    RoomVisibility, ensure_member_removable,
};
pub use session::{Session, SessionError, SessionId, SessionStatus, SessionVisibility};
pub use status::InvalidStatus;
pub use user::{User, UserError, UserStatus};
pub use validation::{
    AvatarUrl, DisplayName, EmailAddress, Issuer, MAX_AVATAR_URL_LEN, MAX_DISPLAY_NAME_LEN,
    MAX_EMAIL_LEN, MAX_ISSUER_LEN, MAX_ROOM_TITLE_LEN, MAX_SESSION_DURATION_MINUTES,
    MAX_SUBJECT_LEN, MAX_USERNAME_LEN, RoomTitle, SessionDurationMinutes, SessionTitle, Subject,
    Username, ValidationError,
};

pub use recording_export::{RecordingExport, RecordingExportStatus};
