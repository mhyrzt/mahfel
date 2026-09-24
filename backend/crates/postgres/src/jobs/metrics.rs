use super::JobStore;

impl JobStore {
    pub async fn pending_stats(&self) -> Result<(i64, f64), sqlx::Error> {
        let row: (i64, f64) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT, COALESCE(EXTRACT(EPOCH FROM \
             (clock_timestamp() - MIN(available_at))),0)::FLOAT8 \
             FROM mahfel.job_outbox WHERE published_at IS NULL AND available_at <= clock_timestamp()",
        ).fetch_one(&self.pool).await?;
        Ok(row)
    }
}
