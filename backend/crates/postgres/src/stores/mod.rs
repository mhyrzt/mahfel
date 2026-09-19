mod identity;
mod room;
mod session;
mod user;

pub use identity::PostgresIdentityStore;
pub use room::PostgresRoomStore;
pub use session::PostgresSessionStore;
pub use user::PostgresUserStore;

use mahfel_application::ports::StoreError;
use sqlx::PgPool;

/// Every PostgreSQL-backed store sharing one connection pool.
///
/// Construct with [`PostgresStores::new`] so composition roots do not each
/// have to know the full set of stores; adding a store is a one-line change
/// here. The bundle itself is not `Clone`; callers take or clone the
/// individual store they need, and each store clone is a cheap pool-handle
/// clone.
pub struct PostgresStores {
    pub users: PostgresUserStore,
    pub identities: PostgresIdentityStore,
    pub rooms: PostgresRoomStore,
    pub sessions: PostgresSessionStore,
}

impl PostgresStores {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            users: PostgresUserStore::new(pool.clone()),
            identities: PostgresIdentityStore::new(pool.clone()),
            rooms: PostgresRoomStore::new(pool.clone()),
            sessions: PostgresSessionStore::new(pool),
        }
    }
}

/// Convert any adapter-level failure into the application-facing store error.
///
/// Infrastructure details stay in the adapter; the application sees only a
/// generic storage failure.
pub(crate) fn store_error(err: impl std::fmt::Display) -> StoreError {
    StoreError::Unexpected(err.to_string())
}

/// Map a `sqlx` failure, preserving known constraint failures as typed variants.
///
/// A foreign-key violation (`23503`) means a referenced row is missing, which
/// callers can turn into a client-visible not-found rather than a 500.
pub(crate) fn sqlx_store_error(err: sqlx::Error) -> StoreError {
    if let sqlx::Error::Database(database) = &err
        && database.code().as_deref() == Some("23503")
    {
        return StoreError::MissingReference(err.to_string());
    }
    StoreError::Unexpected(err.to_string())
}
