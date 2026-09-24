//! PostgreSQL authority for background jobs and their transactional outbox.

mod enqueue;
mod execution;
mod metrics;
mod model;
mod operator;
mod outbox;

pub use enqueue::insert_job_in_tx;
pub use model::{JobAttempt, JobClaim, JobRecord, JobReplay, LeasedOutbox};

use sqlx::PgPool;

#[derive(Clone)]
pub struct JobStore {
    pool: PgPool,
}

impl JobStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
