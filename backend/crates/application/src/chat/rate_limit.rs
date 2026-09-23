use crate::{chat::ChatOperationError, ports::CoordinationStore};
use mahfel_domain::{ActorId, RoomId};
use sha2::{Digest, Sha256};
use std::time::Duration;

/// Shared fixed windows bound message bursts across API replicas. A failed
/// coordination call fails closed; it cannot silently disable flood control.
pub async fn check_chat_post_rate<C: CoordinationStore + ?Sized>(
    coordination: &C,
    room_id: RoomId,
    actor_id: ActorId,
) -> Result<(), ChatOperationError> {
    let mut digest = Sha256::new();
    digest.update(room_id.as_uuid().as_bytes());
    digest.update(actor_id.as_uuid().as_bytes());
    let opaque = digest.finalize();
    let mut key_suffix = String::with_capacity(64);
    for byte in opaque {
        key_suffix.push_str(&format!("{byte:02x}"));
    }
    let burst_key = format!("chat-post:burst:{key_suffix}");
    let minute_key = format!("chat-post:minute:{key_suffix}");
    let burst = coordination
        .increment_with_ttl(&burst_key, Duration::from_secs(10))
        .await?;
    let minute = coordination
        .increment_with_ttl(&minute_key, Duration::from_secs(60))
        .await?;
    if burst > 10 || minute > 60 {
        return Err(ChatOperationError::RateLimited);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{BoxFuture, CoordinationError};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Counts(Mutex<std::collections::HashMap<String, u64>>);

    impl CoordinationStore for Counts {
        fn set_with_ttl<'a>(
            &'a self,
            _: &'a str,
            _: &'a [u8],
            _: Duration,
        ) -> BoxFuture<'a, Result<(), CoordinationError>> {
            Box::pin(async { Ok(()) })
        }
        fn get<'a>(
            &'a self,
            _: &'a str,
        ) -> BoxFuture<'a, Result<Option<Vec<u8>>, CoordinationError>> {
            Box::pin(async { Ok(None) })
        }
        fn delete<'a>(&'a self, _: &'a str) -> BoxFuture<'a, Result<(), CoordinationError>> {
            Box::pin(async { Ok(()) })
        }
        fn publish<'a>(
            &'a self,
            _: &'a str,
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<(), CoordinationError>> {
            Box::pin(async { Ok(()) })
        }
        fn increment_with_ttl<'a>(
            &'a self,
            key: &'a str,
            _: Duration,
        ) -> BoxFuture<'a, Result<u64, CoordinationError>> {
            Box::pin(async move {
                let mut counts = self.0.lock().expect("counter lock");
                let count = counts.entry(key.to_owned()).or_default();
                *count += 1;
                Ok(*count)
            })
        }
    }

    #[tokio::test]
    async fn concurrent_posts_share_one_room_actor_budget() {
        let counts = Arc::new(Counts::default());
        let room = RoomId::new();
        let actor = ActorId::new();
        let mut tasks = tokio::task::JoinSet::new();
        for _ in 0..20 {
            let counts = Arc::clone(&counts);
            tasks.spawn(async move { check_chat_post_rate(counts.as_ref(), room, actor).await });
        }
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            results.push(result.expect("rate limit task"));
        }
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 10);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err(ChatOperationError::RateLimited)))
                .count(),
            10
        );
        assert!(
            check_chat_post_rate(counts.as_ref(), room, ActorId::new())
                .await
                .is_ok(),
            "another actor has an independent budget"
        );
    }
}
