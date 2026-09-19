use crate::http::{auth::AuthState, ws::WsState};
use mahfel_application::ports::RoomStore;

/// Store bound required by the API transport.
///
/// [`RoomStore`] uses return-position `impl Future`, so it is not
/// dyn-compatible; the router state is generic over the concrete adapter
/// instead of holding a boxed trait object.
pub trait RoomStoreState: RoomStore + Clone + Send + Sync + 'static {}

impl<T> RoomStoreState for T where T: RoomStore + Clone + Send + Sync + 'static {}

/// Shared router state for the product API.
#[derive(Clone)]
pub struct AppState<R> {
    pub service: &'static str,
    pub version: &'static str,
    pub node_id: String,
    pub auth: AuthState,
    pub rooms: R,
    pub realtime: WsState,
}
