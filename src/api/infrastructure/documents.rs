//! Object storage for uploaded PDFs.
//!
//! Files do not pass through the API: the browser sends them to storage with a
//! short-lived grant that fixes the key, the content type and the maximum size,
//! and reads them back through a short-lived signed link. The one exception is
//! the hosted demo, where the browser has no route to storage and the API
//! carries small files itself (`put`, `get`). Keys always start with the
//! tenant's prefix, and every operation here checks that.
use crate::{
    dependency_metrics::{OBJECT_STORE, measured},
    error::{AppError, Result, storage},
};
use forklaunch_core::config;
use forklaunch_infrastructure_s3::{
    AsyncObjectStore, ObjectMetadata,
    aws::{AwsS3ObjectStore, PresignDownloadOptions, PresignUploadOptions},
};
use shared::storage::{MAX_UPLOAD_BYTES, tenant_prefix};
use std::{collections::BTreeMap, time::Duration};
use uuid::Uuid;

/// Classification recorded on every stored document (database row and object tag).
pub const DOCUMENT_CLASSIFICATION: &str = "pii";
const UPLOAD_GRANT: Duration = Duration::from_secs(300);
const DOWNLOAD_GRANT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct DocumentStore {
    store: AwsS3ObjectStore,
    tenant: Uuid,
    /// The endpoint the API itself uses, and what browsers should use instead
    /// (for example a same-origin proxy path). Signed links are rewritten from
    /// the first to the second.
    endpoint: Option<String>,
    public_base: Option<String>,
    /// Whether server-side writes also label the object with its
    /// classification. Off (`S3_OBJECT_TAGS=false`) where the storage role may
    /// not tag objects; the classification is always on the database row.
    tag_objects: bool,
}

pub struct UploadTarget {
    pub url: String,
    pub fields: BTreeMap<String, String>,
}

pub struct StoredHead {
    pub size: u64,
    pub is_pdf: bool,
}

impl DocumentStore {
    /// Reads `S3_BUCKET`, `S3_REGION` and, for local or S3-compatible storage,
    /// `S3_URL` with `S3_ACCESS_KEY_ID`/`S3_SECRET_ACCESS_KEY`. Without static
    /// credentials the AWS default chain (task role) is used.
    pub async fn from_env(tenant: Uuid) -> Result<Self> {
        let mut environment =
            forklaunch_infrastructure_s3::aws::environment::S3Environment::from_env()
                .map_err(storage("read object store settings"))?;
        let endpoint = environment.endpoint().map(|url| url.trim_end_matches('/').to_owned());
        environment.options.max_object_bytes =
            usize::try_from(MAX_UPLOAD_BYTES).unwrap_or(usize::MAX);
        // Every storage call has a deadline; the SDK retries transient failures
        // a bounded number of times with jittered exponential backoff.
        environment.options.operation_timeout = Duration::from_secs(20);
        let store = environment.into_store().await.map_err(storage("configure object store"))?;
        Ok(Self {
            store,
            tenant,
            endpoint,
            public_base: config::optional::<String>("S3_PUBLIC_URL")?
                .map(|url| url.trim_end_matches('/').to_owned())
                .filter(|url| !url.is_empty()),
            tag_objects: config::optional::<bool>("S3_OBJECT_TAGS")?.unwrap_or(true),
        })
    }

    fn check_tenant(&self, key: &str) -> Result<()> {
        if key.starts_with(&tenant_prefix(self.tenant)) && !key.contains("..") {
            Ok(())
        } else {
            tracing::error!(
                event = "cross_tenant_storage_key",
                "storage key is outside the tenant prefix"
            );
            Err(AppError::NotFound)
        }
    }

    fn browser_url(&self, url: &str) -> String {
        match (&self.endpoint, &self.public_base) {
            (Some(endpoint), Some(public)) => match url.strip_prefix(endpoint.as_str()) {
                Some(rest) => format!("{public}{rest}"),
                None => url.to_owned(),
            },
            _ => url.to_owned(),
        }
    }

    pub async fn upload_target(&self, key: &str) -> Result<UploadTarget> {
        self.check_tenant(key)?;
        let options = PresignUploadOptions {
            content_type: "application/pdf".to_owned(),
            max_bytes: MAX_UPLOAD_BYTES,
            expires_in: Some(UPLOAD_GRANT),
        };
        let grant =
            measured(OBJECT_STORE, "presign_upload", self.store.presign_upload(key, &options))
                .await
                .map_err(storage("sign upload grant"))?;
        Ok(UploadTarget { url: self.browser_url(&grant.url), fields: grant.fields })
    }

    pub async fn download_url(&self, key: &str, filename: &str) -> Result<String> {
        self.check_tenant(key)?;
        let options = PresignDownloadOptions {
            expires_in: Some(DOWNLOAD_GRANT),
            filename: Some(filename.to_owned()),
        };
        let url =
            measured(OBJECT_STORE, "presign_download", self.store.presign_download(key, &options))
                .await
                .map_err(storage("sign download link"))?;
        Ok(self.browser_url(&url))
    }

    /// Size and file signature of a stored object, reading only its first chunk.
    pub async fn head(&self, key: &str) -> Result<Option<StoredHead>> {
        self.check_tenant(key)?;
        let Some(mut download) = measured(OBJECT_STORE, "head", self.store.download(key))
            .await
            .map_err(storage("read stored object"))?
        else {
            return Ok(None);
        };
        let first =
            download.next_chunk().await.map_err(storage("read stored object"))?.unwrap_or_default();
        Ok(Some(StoredHead {
            size: download.content_length.unwrap_or(first.len() as u64),
            is_pdf: first.starts_with(b"%PDF-"),
        }))
    }

    /// The whole stored object, for the hosted demo's download through the
    /// API. Refuses anything larger than `limit`.
    pub async fn get(&self, key: &str, limit: u64) -> Result<Option<Vec<u8>>> {
        self.check_tenant(key)?;
        let Some(mut download) = measured(OBJECT_STORE, "get", self.store.download(key))
            .await
            .map_err(storage("read stored object"))?
        else {
            return Ok(None);
        };
        let too_large = || AppError::Storage("stored object exceeds the download limit".into());
        if download.content_length.is_some_and(|length| length > limit) {
            return Err(too_large());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) =
            download.next_chunk().await.map_err(storage("read stored object"))?
        {
            if (bytes.len() + chunk.len()) as u64 > limit {
                return Err(too_large());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Some(bytes))
    }

    /// Server-side write, used for demo seeding, the legacy file migration and
    /// the hosted demo's upload through the API.
    pub async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<()> {
        self.check_tenant(key)?;
        let mut metadata = ObjectMetadata::new().with_content_type("application/pdf");
        if self.tag_objects {
            metadata = metadata.with_tag("classification", DOCUMENT_CLASSIFICATION);
        }
        measured(OBJECT_STORE, "put", self.store.put(key, bytes, metadata))
            .await
            .map_err(storage("store object"))?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        self.check_tenant(key)?;
        measured(OBJECT_STORE, "delete", self.store.delete(key))
            .await
            .map_err(storage("delete stored object"))?;
        Ok(())
    }

    /// Readiness probe: one bounded request for a key that does not exist.
    pub async fn ping(&self) -> Result<()> {
        let probe = format!("{}.readiness-probe", tenant_prefix(self.tenant));
        measured(OBJECT_STORE, "ping", async {
            tokio::time::timeout(Duration::from_secs(2), self.store.download(&probe))
                .await
                .map_err(|_| AppError::Storage("object store readiness timed out".into()))?
                .map_err(storage("object store readiness"))
        })
        .await?;
        Ok(())
    }
}
