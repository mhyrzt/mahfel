#[derive(Clone)]
pub struct S3Settings {
    pub internal_endpoint: String,
    pub public_endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub force_path_style: bool,
}

impl From<&mahfel_config::StorageSettings> for S3Settings {
    fn from(settings: &mahfel_config::StorageSettings) -> Self {
        Self {
            internal_endpoint: settings.internal_endpoint.clone(),
            public_endpoint: settings.public_endpoint.clone(),
            region: settings.region.clone(),
            bucket: settings.bucket.clone(),
            access_key: settings.access_key.clone(),
            secret_key: settings.secret_key.clone(),
            force_path_style: settings.force_path_style,
        }
    }
}
