//! Documents go straight to object storage: the API issues a short-lived,
//! size-capped upload grant, confirms the stored object, and queues extraction.
use crate::{
    api::policy::CurrentUser,
    domain::schemas::{
        IdPath,
        documents::{DocumentAccepted, DownloadGrant, NewDocument, UploadGrant},
    },
    error::AppError,
    registrations::AppState,
};
use forklaunch_axum::native::{EndpointError, State};
use forklaunch_core::contract::endpoint::{NoFields, RequestFields};

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

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{
        IdPath,
        documents::{DocumentAccepted, DownloadGrant, NewDocument, UploadGrant},
    };
    use forklaunch_core::contract::{
        HttpMethod, RbacConfig,
        endpoint::{Endpoint, EndpointBody, Fields, JsonBody, NoBody, NoFields},
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
}
