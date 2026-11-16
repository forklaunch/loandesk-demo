//! Access to the document store (S3 or S3-compatible), from the same `S3_*`
//! settings the API uses: the worker reads uploaded files and, in the
//! scheduled retention job, deletes the files of purged records.
use forklaunch_core::{
    common::{FrameworkError, FrameworkResult},
    config,
};
use forklaunch_infrastructure_s3::aws::{AwsS3ObjectStore, environment::S3Environment};
use shared::storage::MAX_UPLOAD_BYTES;
use std::time::Duration;

/// Documents are personal data: outside local development the store is only
/// reached over HTTPS. `None` is AWS S3 through the SDK's own HTTPS endpoints.
pub fn endpoint_allowed(url: Option<&str>, plaintext_development: bool) -> bool {
    plaintext_development || url.is_none_or(|url| url.starts_with("https://"))
}

pub async fn object_store() -> FrameworkResult<AwsS3ObjectStore> {
    let plaintext_development =
        config::required::<String>("WORKER_ALLOW_PLAINTEXT_DEVELOPMENT")? == "true";
    let url = config::optional::<String>("S3_URL")?.filter(|url| !url.trim().is_empty());
    if !endpoint_allowed(url.as_deref(), plaintext_development) {
        return Err(FrameworkError::new(
            "insecure_object_store",
            "S3_URL must be an https:// endpoint unless WORKER_ALLOW_PLAINTEXT_DEVELOPMENT=true",
        ));
    }
    let mut environment = S3Environment::from_env()?;
    environment.options.max_object_bytes = usize::try_from(MAX_UPLOAD_BYTES).unwrap_or(usize::MAX);
    // Every storage call has a deadline; a slow store fails the attempt and the
    // job is retried later instead of holding the worker.
    environment.options.operation_timeout = Duration::from_secs(60);
    environment.into_store().await
}

#[cfg(test)]
mod tests {
    use super::endpoint_allowed;

    #[test]
    fn calls_to_the_object_store_use_https_outside_local_development() {
        assert!(endpoint_allowed(None, false), "AWS S3: the SDK's HTTPS endpoint");
        assert!(endpoint_allowed(Some("https://objects.internal.example"), false));
        assert!(!endpoint_allowed(Some("http://object-store:9000"), false));
        assert!(endpoint_allowed(Some("http://object-store:9000"), true), "local Compose network");
    }
}
