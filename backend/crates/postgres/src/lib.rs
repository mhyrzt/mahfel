//! PostgreSQL adapter.
//!
//! Implements application ports and owns SQL mapping, connection pooling,
//! migration running, and pool metrics. Concrete `sqlx` and `prometheus`
//! errors are converted into [`DatabaseError`] at this boundary.

mod collaboration;
mod error;
mod jobs;
mod metrics;
mod migrations;
mod pool;
mod recording;
mod session_lifecycle;
mod settings;
mod stores;

pub use collaboration::PostgresCollaborationStore;
pub use error::DatabaseError;
pub use jobs::{JobAttempt, JobClaim, JobRecord, JobStore, LeasedOutbox, insert_job_in_tx};
pub use metrics::PoolMetrics;
pub use migrations::migrate;
pub use pool::{connect, ping};
pub use recording::{
    CaptureIntent, CaptureObservation, PostgresRecordingStore, RecordingAssetCopy,
};
pub use session_lifecycle::{PostgresSessionLifecycle, SessionLifecycleGuard};
pub use settings::{ConnectionSettings, PoolSettings};
pub use stores::{
    PostgresAdmissionStore, PostgresBlockStore, PostgresChatStore, PostgresFileStore,
    PostgresIdentityStore, PostgresInviteStore, PostgresMediaPlacementStore,
    PostgresParticipantStore, PostgresParticipationStore, PostgresRoomStore,
    PostgresServicePrincipalStore, PostgresSessionStore, PostgresStores, PostgresUserStore,
};

mod interactions;
pub use interactions::PostgresInteractionStore;
