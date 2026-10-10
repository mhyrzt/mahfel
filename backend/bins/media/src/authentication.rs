use subtle::ConstantTimeEq;
use tonic::{Request, Status, service::Interceptor};

#[derive(Clone)]
pub(crate) struct MediaAuthorization {
    token: Vec<u8>,
}

impl MediaAuthorization {
    pub(crate) fn new(token: &str) -> anyhow::Result<Self> {
        anyhow::ensure!(
            token.len() >= 32 && token.is_ascii(),
            "media control token must contain at least 32 ASCII bytes"
        );
        Ok(Self {
            token: token.as_bytes().to_vec(),
        })
    }
}

impl Interceptor for MediaAuthorization {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));
        if !token.is_some_and(|token| bool::from(self.token.ct_eq(token.as_bytes()))) {
            return Err(Status::unauthenticated(
                "media control authentication required",
            ));
        }
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_control_requires_the_configured_credential() {
        let token = "a-local-test-media-control-token-32-bytes";
        let mut interceptor = MediaAuthorization::new(token).expect("valid fixture");
        for value in [None, Some("Bearer wrong"), Some("Basic irrelevant")] {
            let mut request = Request::new(());
            if let Some(value) = value {
                request
                    .metadata_mut()
                    .insert("authorization", value.parse().expect("metadata"));
            }
            assert_eq!(
                interceptor.call(request).expect_err("rejected").code(),
                tonic::Code::Unauthenticated
            );
        }
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {token}").parse().expect("metadata"),
        );
        assert!(interceptor.call(request).is_ok());
        assert!(MediaAuthorization::new("").is_err());
    }
}
