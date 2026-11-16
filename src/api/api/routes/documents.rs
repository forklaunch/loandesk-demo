use crate::{
    api::{
        controllers::documents::{
            complete_upload,
            contract::{Bindings, CompleteUpload, DownloadDocument, ReadContent, RequestUpload},
            download_document, read_content, request_upload,
        },
        policy::SessionCookiePolicy,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    Bindings {
        request_upload: RequestUpload::bind_authorized(SessionCookiePolicy, request_upload),
        complete_upload: CompleteUpload::bind_authorized(SessionCookiePolicy, complete_upload),
        download_document: DownloadDocument::bind_authorized(
            SessionCookiePolicy,
            download_document,
        ),
        read_content: ReadContent::bind_authorized(SessionCookiePolicy, read_content),
    }
    .mount(router)
}
