use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    applications::load_visible,
    auth::{audit, CurrentUser},
    error::{bad, ApiResult, AppError},
    AppState,
};

pub const MAX_UPLOAD_BYTES: usize = 20 * 1024 * 1024;
const DOC_TYPES: [&str; 3] = ["tax_return", "bank_statement", "financial_statement"];

#[derive(Serialize, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub doc_type: String,
    pub filename: String,
    pub size_bytes: i64,
    pub status: String,
    pub error: Option<String>,
    pub revenue: Option<i64>,
    pub net_income: Option<i64>,
    pub total_debt: Option<i64>,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by_name: String,
}

/// The uploaded name is only ever used as a label and in the download
/// header; files on disk are named by a generated UUID.
fn clean_filename(raw: &str) -> String {
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

pub async fn upload(
    State(st): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let app = load_visible(&st, &user, id).await?;
    if app.created_by != user.id {
        return Err(AppError::Forbidden(
            "Only the relationship manager who owns this application can upload documents",
        ));
    }
    if app.status != "submitted" {
        return Err(AppError::Conflict(
            "This application has already been decided",
        ));
    }

    let mut doc_type = None;
    let mut file = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| bad("Could not read the upload"))?
    {
        match field.name() {
            Some("doc_type") => {
                doc_type = Some(field.text().await.map_err(|_| bad("Could not read the upload"))?)
            }
            Some("file") => {
                let filename = clean_filename(field.file_name().unwrap_or(""));
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| bad("The file is too large (limit is 20 MB) or the upload was interrupted"))?;
                file = Some((filename, bytes));
            }
            _ => {}
        }
    }
    let doc_type = doc_type
        .filter(|t| DOC_TYPES.contains(&t.as_str()))
        .ok_or_else(|| bad("Choose a document type"))?;
    let (filename, bytes) = file.ok_or_else(|| bad("Choose a PDF file to upload"))?;
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(bad("The file is too large (limit is 20 MB)"));
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err(bad("Only PDF files can be uploaded"));
    }

    let doc_id = Uuid::new_v4();
    let storage_key = format!("{doc_id}.pdf");
    let path = st.docs_dir.join(&storage_key);
    tokio::fs::write(&path, &bytes).await?;

    let inserted = sqlx::query(
        "INSERT INTO documents (id, application_id, doc_type, filename, size_bytes, storage_key, uploaded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(doc_id)
    .bind(id)
    .bind(&doc_type)
    .bind(&filename)
    .bind(bytes.len() as i64)
    .bind(&storage_key)
    .bind(user.id)
    .execute(&st.db)
    .await;
    if let Err(e) = inserted {
        let _ = tokio::fs::remove_file(&path).await;
        return Err(e.into());
    }
    audit(&st.db, Some(user.id), "document_uploaded", Some(doc_id), None).await?;

    Ok((StatusCode::CREATED, Json(json!({ "id": doc_id }))))
}

pub async fn download(
    State(st): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let doc: Option<(Uuid, String, String)> =
        sqlx::query_as("SELECT application_id, filename, storage_key FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(&st.db)
            .await?;
    let (application_id, filename, storage_key) = doc.ok_or(AppError::NotFound)?;
    // Same access rule as the application the document belongs to.
    load_visible(&st, &user, application_id).await?;

    let bytes = tokio::fs::read(st.docs_dir.join(&storage_key)).await?;
    audit(&st.db, Some(user.id), "document_downloaded", Some(id), None).await?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(bytes),
    )
        .into_response())
}
