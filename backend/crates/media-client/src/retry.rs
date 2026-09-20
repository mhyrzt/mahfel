use tonic::{Code, Status, transport::Channel};

use super::gateway::GrpcMediaGateway;
use mahfel_rtc_protocol::media::media_control_client::MediaControlClient;

impl GrpcMediaGateway {
    pub(super) async fn with_retry<F, T, Fut>(&self, mut operation: F) -> Result<T, Status>
    where
        F: FnMut(MediaControlClient<Channel>) -> Fut + Send,
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
