use super::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, User};

/// Persistence for user actors. Methods return [`BoxFuture`] so the port is
/// dyn-compatible.
pub trait UserStore: Send + Sync {
    fn get(&self, id: ActorId) -> BoxFuture<'_, Result<Option<User>, StoreError>>;

    fn insert<'a>(&'a self, user: &'a User) -> BoxFuture<'a, Result<(), StoreError>>;

    fn save<'a>(&'a self, user: &'a User) -> BoxFuture<'a, Result<(), StoreError>>;
}
