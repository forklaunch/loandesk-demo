//! Document upload, download and hand-off to the extraction worker.
use crate::{
    api::{policy::CurrentUser, transport},
    document_store::{DOCUMENT_CLASSIFICATION, DocumentStore},
    domain::{
        schemas::documents::{DocumentAccepted, DownloadGrant, NewDocument, UploadGrant},
        services::ApplicationsService,
    },
    error::{AppError, Result, bad, storage},
    extraction_queue::ExtractionQueue,
    persistence::{Db, Disclosures},
};
use shared::storage::{MAX_DOCUMENTS_PER_APPLICATION, document_key};
use sqlx::Row;
use uuid::Uuid;

const DOC_TYPES: [&str; 3] = ["tax_return", "bank_statement", "financial_statement"];
const NOT_A_PDF: &str = "Only PDF files can be uploaded";

/// "The file is too large (limit is 20 MB)", for the limit in force.
pub fn too_large(max_bytes: u64) -> String {
    format!("The file is too large (limit is {} MB)", max_bytes / (1024 * 1024))
}

#[derive(Clone)]
pub struct DocumentsService {
    db: Db,
    applications: ApplicationsService,
    store: DocumentStore,
    queue: ExtractionQueue,
    /// Largest PDF accepted (`Settings::max_upload_bytes`).
    max_bytes: u64,
    /// Files are sent to and read from the API (`Settings::documents_through_api`).
    through_api: bool,
}

/// The uploaded name is only ever used as a label and in the download
/// header; stored objects are named by a generated UUID.
pub fn clean_filename(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let mut name: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' '))
        .take(120)
        .collect();
    name = name.trim().trim_matches('.').to_string();
    if name.is_empty() {
        name = "document".to_string();
    }
    if !name.to_ascii_lowercase().ends_with(".pdf") {
        name.push_str(".pdf");
    }
    name
}

impl DocumentsService {
    pub fn new(
        db: Db,
        applications: ApplicationsService,
        store: DocumentStore,
        queue: ExtractionQueue,
        max_bytes: u64,
        through_api: bool,
    ) -> Self {
        Self { db, applications, store, queue, max_bytes, through_api }
    }

    fn deny(user: &CurrentUser, action: &str, resource: Uuid, message: &str) -> AppError {
        tracing::warn!(
            event = "permission_denied",
            user_id = %user.id,
            role = %user.role,
            action,
            resource = %resource,
            reason = message,
            request_id = %transport::request_id(),
            "permission denied"
        );
        AppError::Forbidden(message.to_owned())
    }

    /// Step 1: record the document and issue a grant to send the file straight
    /// to object storage. Storage itself enforces the PDF content type and the
    /// 20 MB limit, so an oversized body is refused before it is stored. Where
    /// files travel through the API, the grant names the API's own route and
    /// `receive_content` enforces the same two things.
    pub async fn request_upload(
        &self,
        user: &CurrentUser,
        application: &str,
        body: NewDocument,
    ) -> Result<UploadGrant> {
        let application = Uuid::parse_str(application).map_err(|_| AppError::NotFound)?;
        if !DOC_TYPES.contains(&body.doc_type.as_str()) {
            return Err(bad("Choose a document type"));
        }
        if body.size_bytes <= 0 {
            return Err(bad("Choose a PDF file to upload"));
        }
        if u64::try_from(body.size_bytes).unwrap_or(u64::MAX) > self.max_bytes {
            return Err(bad(too_large(self.max_bytes)));
        }
        let filename = clean_filename(&body.filename);

        let mut tx = self.db.tx().await?;
        let app = self.applications.load_visible(&mut tx, user, application).await?;
        if app.created_by != user.id {
            return Err(Self::deny(
                user,
                "upload document",
                application,
                "Only the relationship manager who owns this application can upload documents",
            ));
        }
        if app.status != "submitted" {
            return Err(AppError::Conflict("This application has already been decided".into()));
        }
        // Grants that were never used do not count against the limit.
        sqlx::query(
            "DELETE FROM documents WHERE application_id = $1 AND status = 'uploading'
             AND uploaded_at < now() - interval '1 hour'",
        )
        .bind(application)
        .execute(&mut *tx)
        .await
        .map_err(storage("clear unused upload grants"))?;
        let existing: i64 =
            sqlx::query_scalar("SELECT count(*) FROM documents WHERE application_id = $1")
                .bind(application)
                .fetch_one(&mut *tx)
                .await
                .map_err(storage("count documents"))?;
        if existing >= MAX_DOCUMENTS_PER_APPLICATION {
            return Err(bad(format!(
                "An application can have at most {MAX_DOCUMENTS_PER_APPLICATION} documents"
            )));
        }

        let id = Uuid::new_v4();
        let key = document_key(self.db.tenant(), application, id);
        let sealed_name = self.db.seal(&self.db.members.filename, id, &filename)?;
        sqlx::query(
            "INSERT INTO documents (id, application_id, doc_type, filename_ciphertext, size_bytes,
                                    storage_key, classification, uploaded_by, status, request_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'uploading', $9)",
        )
        .bind(id)
        .bind(application)
        .bind(&body.doc_type)
        .bind(sealed_name.ciphertext())
        .bind(body.size_bytes)
        .bind(&key)
        .bind(DOCUMENT_CLASSIFICATION)
        .bind(user.id)
        .bind(transport::request_id())
        .execute(&mut *tx)
        .await
        .map_err(storage("record document"))?;
        let (url, fields) = if self.through_api {
            (format!("/api/documents/{id}/content"), std::collections::BTreeMap::new())
        } else {
            let target = self.store.upload_target(&key).await?;
            (target.url, target.fields)
        };
        self.db.audit(&mut tx, Some(user.id), "document_upload_started", Some(id), None).await?;
        tx.commit().await.map_err(storage("record document"))?;
        Ok(UploadGrant {
            id: id.to_string(),
            url,
            fields,
            max_bytes: i64::try_from(self.max_bytes).unwrap_or(i64::MAX),
        })
    }

    /// Hosted demo, between steps 1 and 2: the file itself, sent to the API
    /// because the browser has no route to object storage. Only the uploader,
    /// only while the grant is unused, only a PDF within the limit; step 2
    /// then checks the stored object again exactly as it does for a direct
    /// upload.
    pub async fn receive_content(
        &self,
        user: &CurrentUser,
        document: &str,
        bytes: Vec<u8>,
    ) -> Result<DocumentAccepted> {
        if !self.through_api {
            return Err(AppError::NotFound);
        }
        let document = Uuid::parse_str(document).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "SELECT application_id, storage_key, status, uploaded_by FROM documents WHERE id = $1 FOR UPDATE",
        )
        .bind(document)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("load document"))?
        .ok_or(AppError::NotFound)?;
        self.applications.load_visible(&mut tx, user, row.get("application_id")).await?;
        if row.get::<Uuid, _>("uploaded_by") != user.id {
            return Err(Self::deny(
                user,
                "send document content",
                document,
                "Only the uploader can send this file",
            ));
        }
        let status: String = row.get("status");
        if status != "uploading" {
            return Err(AppError::Conflict("This upload has already been completed".into()));
        }
        if let Some(message) = content_rejection(&bytes, self.max_bytes) {
            return Err(bad(message));
        }
        self.store.put(&row.get::<String, _>("storage_key"), bytes).await?;
        tx.commit().await.map_err(storage("store document content"))?;
        Ok(DocumentAccepted { id: document.to_string(), status })
    }

    /// Step 2: check what actually arrived (a PDF within the limit), then mark
    /// the document pending and queue extraction in one transaction. Calling it
    /// again for the same document changes nothing and queues nothing.
    pub async fn complete_upload(
        &self,
        user: &CurrentUser,
        document: &str,
    ) -> Result<DocumentAccepted> {
        let document = Uuid::parse_str(document).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "SELECT application_id, storage_key, status, uploaded_by FROM documents WHERE id = $1 FOR UPDATE",
        )
        .bind(document)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("load document"))?
        .ok_or(AppError::NotFound)?;
        let application: Uuid = row.get("application_id");
        let key: String = row.get("storage_key");
        let status: String = row.get("status");
        self.applications.load_visible(&mut tx, user, application).await?;
        if row.get::<Uuid, _>("uploaded_by") != user.id {
            return Err(Self::deny(
                user,
                "complete upload",
                document,
                "Only the uploader can complete this upload",
            ));
        }
        if status != "uploading" {
            return Ok(DocumentAccepted { id: document.to_string(), status });
        }

        let Some(head) = self.store.head(&key).await? else {
            return Err(bad("The upload did not arrive. Try again."));
        };
        let rejection = if head.size > self.max_bytes {
            Some(too_large(self.max_bytes))
        } else if !head.is_pdf {
            Some(NOT_A_PDF.to_owned())
        } else {
            None
        };
        if let Some(message) = rejection {
            sqlx::query("DELETE FROM documents WHERE id = $1")
                .bind(document)
                .execute(&mut *tx)
                .await
                .map_err(storage("discard rejected upload"))?;
            self.db
                .audit(&mut tx, Some(user.id), "document_rejected", Some(document), None)
                .await?;
            tx.commit().await.map_err(storage("discard rejected upload"))?;
            // The record is gone; remove the stored bytes with it.
            self.store.delete(&key).await?;
            return Err(bad(message));
        }

        sqlx::query("UPDATE documents SET status = 'pending', size_bytes = $2, uploaded_at = now() WHERE id = $1")
            .bind(document)
            .bind(i64::try_from(head.size).unwrap_or(i64::MAX))
            .execute(&mut *tx)
            .await
            .map_err(storage("accept document"))?;
        self.queue.enqueue(&mut tx, document, application).await?;
        self.db.audit(&mut tx, Some(user.id), "document_uploaded", Some(document), None).await?;
        tx.commit().await.map_err(storage("accept document"))?;
        tracing::info!(
            event = "document_uploaded",
            document = %document,
            application = %application,
            user_id = %user.id,
            request_id = %transport::request_id(),
            "document accepted and queued for extraction"
        );
        Ok(DocumentAccepted { id: document.to_string(), status: "pending".into() })
    }

    /// A one-minute link to the stored PDF, for people who may see its
    /// application. The download is written to the audit trail first.
    pub async fn download(&self, user: &CurrentUser, document: &str) -> Result<DownloadGrant> {
        if self.through_api {
            // There is no link a browser could follow; `read_content` serves the file.
            return Err(AppError::NotFound);
        }
        let document = Uuid::parse_str(document).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "SELECT application_id, storage_key, filename_ciphertext FROM documents
             WHERE id = $1 AND status <> 'uploading'",
        )
        .bind(document)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("load document"))?
        .ok_or(AppError::NotFound)?;
        // Same access rule as the application the document belongs to.
        self.applications.load_visible(&mut tx, user, row.get("application_id")).await?;
        let disclosures = Disclosures::new();
        let filename = self
            .db
            .reveal(
                &self.db.members.filename,
                document,
                row.get("filename_ciphertext"),
                user.id,
                "download-document",
                &disclosures,
            )
            .await?;
        let url = self.store.download_url(&row.get::<String, _>("storage_key"), &filename).await?;
        self.db.audit(&mut tx, Some(user.id), "document_downloaded", Some(document), None).await?;
        disclosures.persist(&mut tx, user.id).await?;
        tx.commit().await.map_err(storage("record download"))?;
        tracing::info!(
            event = "document_downloaded",
            user_id = %user.id,
            document = %document,
            request_id = %transport::request_id(),
            "document download link issued"
        );
        Ok(DownloadGrant { url })
    }

    /// Hosted demo: the stored PDF itself, for people who may see its
    /// application. As with a download link, the audit entry is committed
    /// before any byte is returned.
    pub async fn read_content(&self, user: &CurrentUser, document: &str) -> Result<Vec<u8>> {
        if !self.through_api {
            return Err(AppError::NotFound);
        }
        let document = Uuid::parse_str(document).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "SELECT application_id, storage_key FROM documents
             WHERE id = $1 AND status <> 'uploading'",
        )
        .bind(document)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("load document"))?
        .ok_or(AppError::NotFound)?;
        // Same access rule as the application the document belongs to.
        self.applications.load_visible(&mut tx, user, row.get("application_id")).await?;
        // Seeded documents are a few kilobytes and uploads are capped, so the
        // whole file fits in one response.
        let bytes = self
            .store
            .get(&row.get::<String, _>("storage_key"), shared::storage::MAX_UPLOAD_BYTES)
            .await?
            .ok_or(AppError::NotFound)?;
        self.db.audit(&mut tx, Some(user.id), "document_downloaded", Some(document), None).await?;
        tx.commit().await.map_err(storage("record download"))?;
        tracing::info!(
            event = "document_downloaded",
            user_id = %user.id,
            document = %document,
            request_id = %transport::request_id(),
            "document sent"
        );
        Ok(bytes)
    }
}

/// Why a file sent through the API is refused, if it is: the same two checks
/// object storage and `complete_upload` apply to a direct upload.
fn content_rejection(bytes: &[u8], max_bytes: u64) -> Option<String> {
    if bytes.len() as u64 > max_bytes {
        Some(too_large(max_bytes))
    } else if !bytes.starts_with(b"%PDF-") {
        Some(NOT_A_PDF.to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{clean_filename, content_rejection, too_large};

    #[test]
    fn a_file_sent_through_the_api_must_be_a_pdf_within_the_limit() {
        let limit = crate::config::HOSTED_DEMO_MAX_UPLOAD_BYTES;
        assert_eq!(too_large(limit), "The file is too large (limit is 4 MB)");
        assert_eq!(
            too_large(shared::storage::MAX_UPLOAD_BYTES),
            "The file is too large (limit is 20 MB)"
        );
        assert_eq!(content_rejection(b"%PDF-1.7 ...", limit), None);
        assert_eq!(
            content_rejection(b"<html>not a pdf</html>", limit).as_deref(),
            Some("Only PDF files can be uploaded")
        );
        assert_eq!(
            content_rejection(b"", limit).as_deref(),
            Some("Only PDF files can be uploaded")
        );
        let mut large = b"%PDF-".to_vec();
        large.resize(usize::try_from(limit).unwrap() + 1, b' ');
        assert_eq!(content_rejection(&large, limit), Some(too_large(limit)));
        large.pop();
        assert_eq!(content_rejection(&large, limit), None, "exactly at the limit");
    }

    #[test]
    fn filenames_are_reduced_to_a_safe_pdf_label() {
        assert_eq!(clean_filename("../../etc/passwd"), "passwd.pdf");
        assert_eq!(clean_filename("C:\\docs\\Tax Return 2025.PDF"), "Tax Return 2025.PDF");
        assert_eq!(clean_filename("\"><script>.pdf"), "script.pdf");
        assert_eq!(clean_filename(""), "document.pdf");
    }
}
