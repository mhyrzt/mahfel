pub mod decide;
pub mod error;
pub mod lobby;
pub mod policy;
pub mod request;

pub use decide::{AdmissionDecision, AdmissionGrant, admit, preflight_session_admit, reject};
pub use error::AdmissionOperationError;
pub use lobby::{Lobby, LobbyEntry, view_lobby, view_room_lobby};
pub use policy::{
    RoomAdmission, SessionAccess, SessionAccessView, set_room_admission, set_session_access,
    view_session_access,
};
pub use request::{AdmissionOutcome, EnqueueAdmission, enqueue_admission, wait_for_admission};
