//! Documents go straight to object storage: the API issues a short-lived,
//! size-capped upload grant, confirms the stored object, and queues extraction.
//! A hosted demo has no route from the browser to storage, so there the two
//! `content` routes carry the file through the API instead.
//!
//! Sending the file (`send_content`) is the one route here that is written by
//! hand instead of declared as a contract: a contract's binary request body is
//! parsed into a JSON array of numbers and capped at 100 KB (framework 0.2.5),
//! which a PDF of a few megabytes cannot go through. It sits inside the same
//! boundaries as every other route, calls the same session policy with its own
//! role restriction, and takes the body only after that policy has admitted
//! the caller.
use crate::{
    api::policy::CurrentUser,
    domain::schemas::{
        IdPath,
        documents::{DocumentAccepted, DownloadGrant, NewDocument, UploadGrant},
    },
    error::AppError,
    registrations::AppState,
};
use axum::{
    Json,
    extract::{Path, Request},
    http::header,
    response::{IntoResponse, Response},
};
use forklaunch_axum::native::{EndpointError, EndpointPolicy, RouteContract, State};
use forklaunch_core::contract::{
    HttpMethod, RbacConfig,
    endpoint::{NoFields, RequestFields},
};

type ById<B> = RequestFields<B, NoFields, NoFields, IdPath>;

pub async fn request_upload(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<NewDocument>,
) -> Result<UploadGrant, EndpointError> {
    state
        .documents
        .request_upload(&user, &request.path.id, request.body)
        .await
        .map_err(AppError::into_endpoint)
}
pub async fn complete_upload(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<()>,
) -> Result<DocumentAccepted, EndpointError> {
    state.documents.complete_upload(&user, &request.path.id).await.map_err(AppError::into_endpoint)
}
pub async fn download_document(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<()>,
) -> Result<DownloadGrant, EndpointError> {
    state.documents.download(&user, &request.path.id).await.map_err(AppError::into_endpoint)
}
/// Where `send_content` is mounted (`routes::documents`), in axum's syntax.
pub const SEND_CONTENT_PATH: &str = "/api/documents/{id}/content";

/// What the session policy needs to know about `send_content`: its name for
/// the log, and that only relationship managers may call it.
pub fn send_content_route() -> RouteContract {
    RouteContract::new(
        "Send Document Content",
        "Hosted demo only: send the PDF (4 MB at most) for an upload grant through the API",
        HttpMethod::Put,
        "/api/documents/:id/content",
    )
    .with_rbac(RbacConfig {
        allowed_roles: vec![crate::api::policy::RELATIONSHIP_MANAGER.to_owned()],
        ..RbacConfig::default()
    })
}

/// `PUT /api/documents/{id}/content` with the PDF as the body.
pub async fn send_content(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let (mut parts, body) = request.into_parts();
    // Session, per-user request limit and role, before a byte of the body is read.
    let user = match crate::api::policy::SessionCookiePolicy
        .authorize(&mut parts, &state, &send_content_route())
        .await
    {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    if !state.limits.expensive.allow(&user.id.to_string()) {
        return AppError::TooManyRequests("Too many requests. Wait a moment and try again.".into())
            .into_endpoint()
            .into_response();
    }
    let declared_pdf = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("application/pdf"));
    if !declared_pdf {
        return EndpointError::UnsupportedContentType.into_response();
    }
    // The framework boundary already limits the body to this size; a longer
    // one fails here and is answered as too large.
    let limit = usize::try_from(state.settings.max_upload_bytes()).unwrap_or(usize::MAX);
    let Ok(bytes) = axum::body::to_bytes(body, limit).await else {
        return AppError::BadRequest(crate::domain::services::too_large(
            state.settings.max_upload_bytes(),
        ))
        .into_endpoint()
        .into_response();
    };
    match state.documents.receive_content(&user, &id, bytes.to_vec()).await {
        Ok(accepted) => Json(accepted).into_response(),
        Err(error) => error.into_endpoint().into_response(),
    }
}
pub async fn read_content(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<()>,
) -> Result<Vec<u8>, EndpointError> {
    state.documents.read_content(&user, &request.path.id).await.map_err(AppError::into_endpoint)
}

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{
        IdPath,
        documents::{DocumentAccepted, DownloadGrant, NewDocument, UploadGrant},
    };
    use forklaunch_core::contract::{
        HttpMethod, MediaType, RbacConfig,
        endpoint::{BinaryBody, Endpoint, EndpointBody, Fields, JsonBody, NoBody, NoFields},
    };

    fn roles<I: EndpointBody, O: EndpointBody>(
        endpoint: Endpoint<I, O>,
        allowed: &[&str],
    ) -> Endpoint<I, O> {
        Endpoint::from_contract(endpoint.into_contract().with_rbac(RbacConfig {
            allowed_roles: allowed.iter().map(|role| (*role).to_owned()).collect(),
            ..RbacConfig::default()
        }))
        .expect("valid role restriction")
    }

    #[access(Protected)]
    pub fn request_upload()
    -> Endpoint<Fields<JsonBody<NewDocument>, NoFields, NoFields, IdPath>, JsonBody<UploadGrant>>
    {
        roles(
            Endpoint::new(
                "Request Document Upload",
                "Issue a grant to upload one PDF (20 MB at most) directly to object storage",
                HttpMethod::Post,
                "/api/applications/:id/documents",
                201,
            )
            .expect("valid request upload endpoint"),
            &["relationship_manager"],
        )
    }
    #[access(Protected)]
    pub fn complete_upload()
    -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, JsonBody<DocumentAccepted>> {
        roles(
            Endpoint::new(
                "Complete Document Upload",
                "Confirm the stored file is a PDF within the size limit and queue figure extraction",
                HttpMethod::Post,
                "/api/documents/:id/complete",
                200,
            )
            .expect("valid complete upload endpoint"),
            &["relationship_manager"],
        )
    }
    #[access(Protected)]
    pub fn download_document()
    -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, JsonBody<DownloadGrant>> {
        roles(
            Endpoint::new(
                "Download Document",
                "Issue a short-lived download link; the download is audited",
                HttpMethod::Post,
                "/api/documents/:id/download-url",
                200,
            )
            .expect("valid download endpoint"),
            &["relationship_manager", "credit_officer"],
        )
    }
    #[access(Protected)]
    pub fn read_content() -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, BinaryBody> {
        roles(
            Endpoint::new(
                "Read Document Content",
                "Hosted demo only: the stored PDF itself; the download is audited",
                HttpMethod::Get,
                "/api/documents/:id/content",
                200,
            )
            .expect("valid read content endpoint"),
            &["relationship_manager", "credit_officer"],
        )
        // Media and fixed headers are declared after the role restriction,
        // which rebuilds the endpoint from its default contract.
        .with_response_media(MediaType::Pdf)
        .and_then(|endpoint| endpoint.with_response_header("content-disposition", "attachment"))
        .expect("PDF attachment response")
    }
}
