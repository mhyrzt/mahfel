use mahfel_application::{
    files::{FileOperationError, cleanup_file},
    jobs::{FILE_CLEANUP_KIND, FileCleanupPayload, JOB_SCHEMA_VERSION, JobEnvelope, JobFailure},
};
use mahfel_domain::FileId;
use mahfel_postgres::PostgresFileStore;
use mahfel_storage::S3FileStorage;

pub(super) async fn handle(
    job: &JobEnvelope,
    files: &PostgresFileStore,
    storage: &S3FileStorage,
) -> Result<(), JobFailure> {
    if job.version != JOB_SCHEMA_VERSION || job.job_type != FILE_CLEANUP_KIND {
        return Err(JobFailure::Permanent("unsupported_job_contract"));
    }
    let payload: FileCleanupPayload = serde_json::from_value(job.payload.clone())
        .map_err(|_| JobFailure::Permanent("invalid_payload"))?;
    cleanup_file(files, storage, FileId::from_uuid(payload.file_id))
        .await
        .map_err(|error| match error {
            FileOperationError::NotReady | FileOperationError::Invalid(_) => {
                JobFailure::Permanent("invalid_file_state")
            }
            _ => JobFailure::Retryable("file_cleanup_unavailable"),
        })
}
