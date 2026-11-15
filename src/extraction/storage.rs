//! Read access to the document store (S3 or S3-compatible), from the same
//! `S3_*` settings the API uses.
use forklaunch_core::common::FrameworkResult;
use forklaunch_infrastructure_s3::aws::{AwsS3ObjectStore, environment::S3Environment};
use shared::storage::MAX_UPLOAD_BYTES;
use std::time::Duration;

pub async fn object_store() -> FrameworkResult<AwsS3ObjectStore> {
    let mut environment = S3Environment::from_env()?;
    environment.options.max_object_bytes = usize::try_from(MAX_UPLOAD_BYTES).unwrap_or(usize::MAX);
    // Every storage call has a deadline; a slow store fails the attempt and the
    // job is retried later instead of holding the worker.
    environment.options.operation_timeout = Duration::from_secs(60);
    environment.into_store().await
}
