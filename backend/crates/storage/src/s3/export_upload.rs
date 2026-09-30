pub(super) struct MultipartGuard {
    client: aws_sdk_s3::Client,
    bucket: String,
    key: String,
    upload_id: String,
    complete: bool,
}
impl MultipartGuard {
    pub(super) fn new(
        client: aws_sdk_s3::Client,
        bucket: String,
        key: String,
        upload_id: String,
    ) -> Self {
        Self {
            client,
            bucket,
            key,
            upload_id,
            complete: false,
        }
    }
    pub(super) fn complete(mut self) {
        self.complete = true;
    }
}
impl Drop for MultipartGuard {
    fn drop(&mut self) {
        if self.complete {
            return;
        }
        let (client, bucket, key, upload_id) = (
            self.client.clone(),
            self.bucket.clone(),
            self.key.clone(),
            self.upload_id.clone(),
        );
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let _ = client
                    .abort_multipart_upload()
                    .bucket(bucket)
                    .key(key)
                    .upload_id(upload_id)
                    .send()
                    .await;
            });
        }
    }
}
