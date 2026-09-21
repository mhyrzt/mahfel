pub mod archive;
pub mod create;
pub mod error;
pub mod get;
pub(crate) mod guard;
pub mod join;
pub mod list;
pub mod members;
pub mod update;

pub use archive::archive_room;
pub use create::{CreateRoom, create_room};
pub use error::RoomOperationError;
pub use get::get_room;
pub use join::join_room;
pub use list::list_joined;
pub use members::{add_member, remove_member};
pub use update::{UpdateRoom, update_settings};

pub(crate) use guard::require_manager;
