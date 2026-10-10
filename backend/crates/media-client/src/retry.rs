use tonic::{Code, Status};

use super::gateway::GrpcMediaGateway;
use super::gateway::MediaControlClient;

impl GrpcMediaGateway {
    /// Run a resource-creating operation once: a lost response must not create a duplicate.
    pub(super) async fn once<F, T, Fut>(&self, operation: F) -> Result<T, Status>
    where
        F: FnOnce(MediaControlClient) -> Fut + Send,
        Fut: std::future::Future<Output = Result<tonic::Response<T>, Status>> + Send,
        T: Send,
    {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            operation(self.client.clone()),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("media gateway timeout"))?
        .map(tonic::Response::into_inner)
    }

    pub(super) async fn with_retry<F, T, Fut>(&self, mut operation: F) -> Result<T, Status>
    where
        F: FnMut(MediaControlClient) -> Fut + Send,
        Fut: std::future::Future<Output = Result<tonic::Response<T>, Status>> + Send,
        T: Send,
    {
        let mut last_status: Option<Status> = None;
        for attempt in 0..3 {
            let client = self.client.clone();
            let fut = operation(client);
            let res = tokio::time::timeout(std::time::Duration::from_secs(5), fut).await;
            match res {
                Ok(Ok(response)) => return Ok(response.into_inner()),
                Ok(Err(status)) => {
                    if status.code() == Code::Unavailable && attempt < 2 {
                        last_status = Some(status);
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << attempt)))
                            .await;
                        continue;
                    }
                    return Err(status);
                }
                Err(_) => {
                    let status = Status::deadline_exceeded("media gateway timeout");
                    if attempt < 2 {
                        last_status = Some(status);
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << attempt)))
                            .await;
                        continue;
                    }
                    return Err(status);
                }
            }
        }
        Err(last_status.unwrap_or_else(|| Status::unavailable("media unavailable")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn gateway() -> GrpcMediaGateway {
        GrpcMediaGateway::lazy("http://127.0.0.1:1".to_owned()).expect("valid local endpoint")
    }

    #[tokio::test]
    async fn resource_creation_does_not_retry_an_ambiguous_failure() {
        let attempts = AtomicUsize::new(0);
        let result: Result<(), Status> = gateway()
            .once(|_| async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err(Status::unavailable("response lost after resource creation"))
            })
            .await;
        assert_eq!(result.expect_err("unavailable").code(), Code::Unavailable);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn single_attempt_operations_have_a_deadline() {
        let result: Result<(), Status> = gateway().once(|_| std::future::pending()).await;
        assert_eq!(
            result.expect_err("bounded operation").code(),
            Code::DeadlineExceeded
        );
    }
}
