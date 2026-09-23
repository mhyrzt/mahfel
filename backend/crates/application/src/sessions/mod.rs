pub mod cancel;
pub mod details;
pub mod end;
pub mod error;
pub mod get;
pub(crate) mod guard;
pub mod join;
pub mod leave;
pub mod list;
pub mod schedule;
pub mod start;
pub mod visibility;

pub use cancel::{CancelSession, cancel_session};
pub use details::{UpdateSessionDetails, update_session_details};
pub use end::{EndSession, end_session};
pub use error::SessionOperationError;
pub use get::{GetSession, get_session};
pub use join::{JoinOutcome, JoinSession, join_session, preflight_join};
pub use leave::{LeaveSession, leave_session};
pub use list::{ListSessions, list_sessions};
pub use schedule::{ScheduleSession, schedule_session};
pub use start::{StartSession, start_session};
pub use visibility::{GetSessionForCaller, get_session_for_caller};

pub(crate) use guard::{require_joinable, require_not_blocked};

#[cfg(test)]
mod tests;
