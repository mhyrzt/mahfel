use aws_sdk_s3::{
    Client,
    config::{
        BehaviorVersion, Config, Credentials, Region, retry::RetryConfig, timeout::TimeoutConfig,
    },
};
use std::time::Duration;

use crate::S3Settings;

pub(super) fn build(settings: &S3Settings, endpoint: &str) -> Client {
    let credentials = Credentials::new(
        settings.access_key.clone(),
        settings.secret_key.clone(),
        None,
        None,
        "mahfel-storage",
    );

    Client::from_conf(
        Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(settings.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .force_path_style(settings.force_path_style)
            .retry_config(RetryConfig::standard().with_max_attempts(3))
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_secs(20))
                    .build(),
            )
            .build(),
    )
}
