//! PostgreSQL adapter.
//!
//! Implements application ports and owns SQL mapping, connection pooling,
//! migration running, and pool metrics. Concrete `sqlx` and `prometheus`
//! errors are converted into [`DatabaseError`] at this boundary.

mod error;
mod metrics;
mod migrations;
mod pool;
mod settings;
mod stores;

pub use error::DatabaseError;
pub use metrics::PoolMetrics;
pub use migrations::migrate;
pub use pool::{connect, ping};
pub use settings::{ConnectionSettings, PoolSettings};
pub use stores::{
    PostgresAdmissionStore, PostgresBlockStore, PostgresChatStore, PostgresFileStore,
    PostgresIdentityStore, PostgresInviteStore, PostgresMediaPlacementStore,
    PostgresParticipantStore, PostgresParticipationStore, PostgresRoomStore,
    PostgresServicePrincipalStore, PostgresSessionStore, PostgresStores, PostgresUserStore,
};
