use mahfel_domain::{ActorId, SessionId};
use sqlx::{PgPool, Postgres, Transaction};

/// Serializes one actor's join and last departure across API replicas. The
/// transaction only holds an advisory lock; application stores retain their
/// existing PostgreSQL ownership and uniqueness constraints.
#[derive(Clone)]
pub struct PostgresSessionLifecycle {
    pool: PgPool,
}

pub struct SessionLifecycleGuard {
    _transaction: Transaction<'static, Postgres>,
}

impl PostgresSessionLifecycle {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn lock(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> Result<SessionLifecycleGuard, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let key = format!("session-presence:{session_id}:{actor_id}");
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(key)
            .execute(&mut *transaction)
            .await?;
        Ok(SessionLifecycleGuard {
            _transaction: transaction,
        })
    }
}
