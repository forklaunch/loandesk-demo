use std::fmt;
use forklaunch_universal_sdk as runtime;
use super::models::*;
use super::schemas;
type ClientRuntime = runtime::UniversalSdk;
#[derive(Clone)]
pub struct Client {
    runtime: ClientRuntime,
}
impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Client").field("runtime", &self.runtime).finish()
    }
}
impl Client {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            runtime: ClientRuntime::new(base_url),
        }
    }
    /// Custom transports must enforce TLS verification and disable credential-bearing redirects.
    #[must_use]
    pub fn with_http_client(base_url: impl Into<String>, http: reqwest::Client) -> Self {
        Self {
            runtime: ClientRuntime::with_http_client(base_url, http),
        }
    }
    /// Local development only: permits plaintext credentials outside loopback.
    #[must_use]
    pub fn with_plaintext_development_transport(mut self) -> Self {
        self.runtime = self.runtime.with_plaintext_development_transport();
        self
    }
    #[must_use]
    pub fn with_default_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.runtime = self.runtime.with_default_header(name, value);
        self
    }
    #[must_use]
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.runtime = self.runtime.with_bearer_token(token);
        self
    }
    #[must_use]
    pub fn with_jwt(self, token: impl Into<String>) -> Self {
        self.with_bearer_token(token)
    }
    #[must_use]
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.runtime = self.runtime.with_basic_auth(username, password);
        self
    }
    /// One fixed `authorization` value; an HMAC credential verifies at most once.
    /// Prefer `with_hmac_secret` for service-to-service calls.
    #[must_use]
    pub fn with_hmac_authorization(mut self, value: impl Into<String>) -> Self {
        self.runtime = self.runtime.with_hmac_authorization(value);
        self
    }
    /// Sign every request with the ForkLaunch request HMAC (`keyId=default`): the final
    /// method, URL path, body bytes, a fresh timestamp and a random nonce.
    #[must_use]
    pub fn with_hmac_secret(mut self, secret: impl Into<Vec<u8>>) -> Self {
        self.runtime = self.runtime.with_hmac_secret(secret);
        self
    }
    /// Sign every request with a named server key.
    #[must_use]
    pub fn with_hmac_signer(mut self, signer: HmacSigner) -> Self {
        self.runtime = self.runtime.with_hmac_signer(signer);
        self
    }
    #[must_use]
    pub fn with_auth_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(&mut SdkRequest) + Send + Sync + 'static,
    {
        self.runtime = self.runtime.with_auth_hook(hook);
        self
    }
    /// Construct a transport request.
    pub fn list_applications_request(
        &self,
        input: ListApplicationsRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params("/api/applications", &input.params);
        Ok(SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: runtime::request_fields(&input.query)?,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn list_applications(
        &self,
        input: ListApplicationsRequest,
    ) -> Result<ListApplicationsResponses, SdkError> {
        let request = self.list_applications_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let headers = runtime::decode_response_headers::<
                    ListApplicationsResponseHeaders,
                >(response.headers(), schemas::LIST_APPLICATIONS_STATUS200_HEADERS)?;
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(
                    ListApplicationsResponses::Status200(SdkResponseWithHeaders {
                        body,
                        headers,
                    }),
                )
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListApplicationsResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn search_applications_request(
        &self,
        input: SearchApplicationsRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        Ok(
            self
                .search_applications_raw_request(SearchApplicationsRequest::<String> {
                    params: input.params,
                    query: input.query,
                    headers: input.headers,
                    body: input
                        .body
                        .map(|body| serde_json::to_string(&body))
                        .transpose()?,
                }),
        )
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn search_applications(
        &self,
        input: SearchApplicationsRequest,
    ) -> Result<SearchApplicationsResponses, SdkError> {
        let input = SearchApplicationsRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.search_applications_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    #[must_use]
    pub fn search_applications_raw_request(
        &self,
        input: SearchApplicationsRequest<String>,
    ) -> SdkRequest {
        let path = runtime::path_with_params("/api/applications/search", &input.params);
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    pub async fn search_applications_raw(
        &self,
        input: SearchApplicationsRequest<String>,
    ) -> Result<SearchApplicationsResponses, SdkError> {
        let request = self.search_applications_raw_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let headers = runtime::decode_response_headers::<
                    SearchApplicationsResponseHeaders,
                >(response.headers(), schemas::LIST_APPLICATIONS_STATUS200_HEADERS)?;
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(
                    SearchApplicationsResponses::Status200(SdkResponseWithHeaders {
                        body,
                        headers,
                    }),
                )
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SearchApplicationsResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn create_application_request(
        &self,
        input: CreateApplicationRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        self.create_application_raw_request(CreateApplicationRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input.body.map(|body| serde_json::to_string(&body)).transpose()?,
        })
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn create_application(
        &self,
        input: CreateApplicationRequest,
    ) -> Result<CreateApplicationResponses, SdkError> {
        let input = CreateApplicationRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.create_application_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    pub fn create_application_raw_request(
        &self,
        input: CreateApplicationRequest<String>,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params("/api/applications", &input.params);
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: runtime::request_fields(&input.headers)?,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn create_application_raw(
        &self,
        input: CreateApplicationRequest<String>,
    ) -> Result<CreateApplicationResponses, SdkError> {
        let request = self
            .create_application_raw_request(input)
            .map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            201 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::CREATE_APPLICATION_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status201(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateApplicationResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn get_application_request(
        &self,
        input: GetApplicationRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/applications/:id",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn get_application(
        &self,
        input: GetApplicationRequest,
    ) -> Result<GetApplicationResponses, SdkError> {
        let request = self.get_application_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_APPLICATION_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetApplicationResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn decide_application_request(
        &self,
        input: DecideApplicationRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        self.decide_application_raw_request(DecideApplicationRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input.body.map(|body| serde_json::to_string(&body)).transpose()?,
        })
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn decide_application(
        &self,
        input: DecideApplicationRequest,
    ) -> Result<DecideApplicationResponses, SdkError> {
        let input = DecideApplicationRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.decide_application_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    pub fn decide_application_raw_request(
        &self,
        input: DecideApplicationRequest<String>,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/applications/:id/decision",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn decide_application_raw(
        &self,
        input: DecideApplicationRequest<String>,
    ) -> Result<DecideApplicationResponses, SdkError> {
        let request = self
            .decide_application_raw_request(input)
            .map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::DECIDE_APPLICATION_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DecideApplicationResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn reveal_owner_ssn_request(
        &self,
        input: RevealOwnerSsnRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/applications/:id/owners/:owner_id/reveal-ssn",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn reveal_owner_ssn(
        &self,
        input: RevealOwnerSsnRequest,
    ) -> Result<RevealOwnerSsnResponses, SdkError> {
        let request = self.reveal_owner_ssn_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::REVEAL_OWNER_SSN_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RevealOwnerSsnResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn export_application_request(
        &self,
        input: ExportApplicationRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/applications/:id/export",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn export_application(
        &self,
        input: ExportApplicationRequest,
    ) -> Result<ExportApplicationResponses, SdkError> {
        let request = self.export_application_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::EXPORT_APPLICATION_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ExportApplicationResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn sign_in_request(
        &self,
        input: SignInRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        Ok(
            self
                .sign_in_raw_request(SignInRequest::<String> {
                    params: input.params,
                    query: input.query,
                    headers: input.headers,
                    body: input
                        .body
                        .map(|body| serde_json::to_string(&body))
                        .transpose()?,
                }),
        )
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn sign_in(
        &self,
        input: SignInRequest,
    ) -> Result<SignInResponses, SdkError> {
        let input = SignInRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.sign_in_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    #[must_use]
    pub fn sign_in_raw_request(&self, input: SignInRequest<String>) -> SdkRequest {
        let path = runtime::path_with_params("/api/auth/login", &input.params);
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    pub async fn sign_in_raw(
        &self,
        input: SignInRequest<String>,
    ) -> Result<SignInResponses, SdkError> {
        let request = self.sign_in_raw_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::SIGN_IN_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignInResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn sign_out_request(&self, input: SignOutRequest) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/auth/logout",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn sign_out(&self) -> Result<SignOutResponses, SdkError> {
        self.sign_out_with_options(SignOutRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn sign_out_with_options(
        &self,
        input: SignOutRequest,
    ) -> Result<SignOutResponses, SdkError> {
        let request = self.sign_out_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::SIGN_OUT_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SignOutResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn current_user_request(&self, input: CurrentUserRequest) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/auth/me",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn current_user(&self) -> Result<CurrentUserResponses, SdkError> {
        self.current_user_with_options(CurrentUserRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn current_user_with_options(
        &self,
        input: CurrentUserRequest,
    ) -> Result<CurrentUserResponses, SdkError> {
        let request = self.current_user_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::SIGN_IN_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CurrentUserResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn start_two_factor_enrolment_request(
        &self,
        input: StartTwoFactorEnrolmentRequest,
    ) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/auth/mfa/enrol",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn start_two_factor_enrolment(
        &self,
    ) -> Result<StartTwoFactorEnrolmentResponses, SdkError> {
        self.start_two_factor_enrolment_with_options(
                StartTwoFactorEnrolmentRequest::default(),
            )
            .await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn start_two_factor_enrolment_with_options(
        &self,
        input: StartTwoFactorEnrolmentRequest,
    ) -> Result<StartTwoFactorEnrolmentResponses, SdkError> {
        let request = self.start_two_factor_enrolment_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::START_TWO_FACTOR_ENROLMENT_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(StartTwoFactorEnrolmentResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn confirm_two_factor_enrolment_request(
        &self,
        input: ConfirmTwoFactorEnrolmentRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        Ok(
            self
                .confirm_two_factor_enrolment_raw_request(ConfirmTwoFactorEnrolmentRequest::<
                    String,
                > {
                    params: input.params,
                    query: input.query,
                    headers: input.headers,
                    body: input
                        .body
                        .map(|body| serde_json::to_string(&body))
                        .transpose()?,
                }),
        )
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn confirm_two_factor_enrolment(
        &self,
        input: ConfirmTwoFactorEnrolmentRequest,
    ) -> Result<ConfirmTwoFactorEnrolmentResponses, SdkError> {
        let input = ConfirmTwoFactorEnrolmentRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.confirm_two_factor_enrolment_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    #[must_use]
    pub fn confirm_two_factor_enrolment_raw_request(
        &self,
        input: ConfirmTwoFactorEnrolmentRequest<String>,
    ) -> SdkRequest {
        let path = runtime::path_with_params("/api/auth/mfa/confirm", &input.params);
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    pub async fn confirm_two_factor_enrolment_raw(
        &self,
        input: ConfirmTwoFactorEnrolmentRequest<String>,
    ) -> Result<ConfirmTwoFactorEnrolmentResponses, SdkError> {
        let request = self.confirm_two_factor_enrolment_raw_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::SIGN_OUT_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ConfirmTwoFactorEnrolmentResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn change_password_request(
        &self,
        input: ChangePasswordRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        Ok(
            self
                .change_password_raw_request(ChangePasswordRequest::<String> {
                    params: input.params,
                    query: input.query,
                    headers: input.headers,
                    body: input
                        .body
                        .map(|body| serde_json::to_string(&body))
                        .transpose()?,
                }),
        )
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn change_password(
        &self,
        input: ChangePasswordRequest,
    ) -> Result<ChangePasswordResponses, SdkError> {
        let input = ChangePasswordRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.change_password_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    #[must_use]
    pub fn change_password_raw_request(
        &self,
        input: ChangePasswordRequest<String>,
    ) -> SdkRequest {
        let path = runtime::path_with_params("/api/auth/password", &input.params);
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    pub async fn change_password_raw(
        &self,
        input: ChangePasswordRequest<String>,
    ) -> Result<ChangePasswordResponses, SdkError> {
        let request = self.change_password_raw_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::SIGN_OUT_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ChangePasswordResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn report_browser_error_request(
        &self,
        input: ReportBrowserErrorRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        Ok(
            self
                .report_browser_error_raw_request(ReportBrowserErrorRequest::<String> {
                    params: input.params,
                    query: input.query,
                    headers: input.headers,
                    body: input
                        .body
                        .map(|body| serde_json::to_string(&body))
                        .transpose()?,
                }),
        )
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn report_browser_error(
        &self,
        input: ReportBrowserErrorRequest,
    ) -> Result<ReportBrowserErrorResponses, SdkError> {
        let input = ReportBrowserErrorRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.report_browser_error_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    #[must_use]
    pub fn report_browser_error_raw_request(
        &self,
        input: ReportBrowserErrorRequest<String>,
    ) -> SdkRequest {
        let path = runtime::path_with_params("/api/client-errors", &input.params);
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    pub async fn report_browser_error_raw(
        &self,
        input: ReportBrowserErrorRequest<String>,
    ) -> Result<ReportBrowserErrorResponses, SdkError> {
        let request = self.report_browser_error_raw_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::SIGN_OUT_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReportBrowserErrorResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn request_document_upload_request(
        &self,
        input: RequestDocumentUploadRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        self.request_document_upload_raw_request(RequestDocumentUploadRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input.body.map(|body| serde_json::to_string(&body)).transpose()?,
        })
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn request_document_upload(
        &self,
        input: RequestDocumentUploadRequest,
    ) -> Result<RequestDocumentUploadResponses, SdkError> {
        let input = RequestDocumentUploadRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.request_document_upload_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    pub fn request_document_upload_raw_request(
        &self,
        input: RequestDocumentUploadRequest<String>,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/applications/:id/documents",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn request_document_upload_raw(
        &self,
        input: RequestDocumentUploadRequest<String>,
    ) -> Result<RequestDocumentUploadResponses, SdkError> {
        let request = self
            .request_document_upload_raw_request(input)
            .map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            201 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::REQUEST_DOCUMENT_UPLOAD_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status201(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RequestDocumentUploadResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn complete_document_upload_request(
        &self,
        input: CompleteDocumentUploadRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/documents/:id/complete",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn complete_document_upload(
        &self,
        input: CompleteDocumentUploadRequest,
    ) -> Result<CompleteDocumentUploadResponses, SdkError> {
        let request = self
            .complete_document_upload_request(input)
            .map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::COMPLETE_DOCUMENT_UPLOAD_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CompleteDocumentUploadResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn download_document_request(
        &self,
        input: DownloadDocumentRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/documents/:id/download-url",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn download_document(
        &self,
        input: DownloadDocumentRequest,
    ) -> Result<DownloadDocumentResponses, SdkError> {
        let request = self.download_document_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::DOWNLOAD_DOCUMENT_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DownloadDocumentResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn read_document_content_request(
        &self,
        input: ReadDocumentContentRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/documents/:id/content",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn read_document_content(
        &self,
        input: ReadDocumentContentRequest,
    ) -> Result<ReadDocumentContentResponses, SdkError> {
        let request = self
            .read_document_content_request(input)
            .map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/pdf" => {
                let headers = runtime::decode_response_headers::<
                    ReadDocumentContentResponseHeaders,
                >(response.headers(), schemas::READ_DOCUMENT_CONTENT_STATUS200_HEADERS)?;
                let body = response.bytes().await?.to_vec();
                Ok(
                    ReadDocumentContentResponses::Status200(SdkResponseWithHeaders {
                        body,
                        headers,
                    }),
                )
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ReadDocumentContentResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn get_health_request(&self, input: GetHealthRequest) -> SdkRequest {
        let path = runtime::path_with_params(
            "/health",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn get_health(&self) -> Result<GetHealthResponses, SdkError> {
        self.get_health_with_options(GetHealthRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn get_health_with_options(
        &self,
        input: GetHealthRequest,
    ) -> Result<GetHealthResponses, SdkError> {
        let request = self.get_health_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn get_readiness_request(&self, input: GetReadinessRequest) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/readyz",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn get_readiness(&self) -> Result<GetReadinessResponses, SdkError> {
        self.get_readiness_with_options(GetReadinessRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn get_readiness_with_options(
        &self,
        input: GetReadinessRequest,
    ) -> Result<GetReadinessResponses, SdkError> {
        let request = self.get_readiness_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetReadinessResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn get_liveness_request(&self, input: GetLivenessRequest) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/healthz",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn get_liveness(&self) -> Result<GetLivenessResponses, SdkError> {
        self.get_liveness_with_options(GetLivenessRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn get_liveness_with_options(
        &self,
        input: GetLivenessRequest,
    ) -> Result<GetLivenessResponses, SdkError> {
        let request = self.get_liveness_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetLivenessResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn get_client_settings_request(
        &self,
        input: GetClientSettingsRequest,
    ) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/client-settings",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn get_client_settings(
        &self,
    ) -> Result<GetClientSettingsResponses, SdkError> {
        self.get_client_settings_with_options(GetClientSettingsRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn get_client_settings_with_options(
        &self,
        input: GetClientSettingsRequest,
    ) -> Result<GetClientSettingsResponses, SdkError> {
        let request = self.get_client_settings_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_CLIENT_SETTINGS_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetClientSettingsResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn list_users_request(
        &self,
        input: ListUsersRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params("/api/users", &input.params);
        Ok(SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: runtime::request_fields(&input.query)?,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn list_users(
        &self,
        input: ListUsersRequest,
    ) -> Result<ListUsersResponses, SdkError> {
        let request = self.list_users_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let headers = runtime::decode_response_headers::<
                    ListUsersResponseHeaders,
                >(response.headers(), schemas::LIST_APPLICATIONS_STATUS200_HEADERS)?;
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_USERS_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(
                    ListUsersResponses::Status200(SdkResponseWithHeaders {
                        body,
                        headers,
                    }),
                )
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListUsersResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn create_user_request(
        &self,
        input: CreateUserRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        Ok(
            self
                .create_user_raw_request(CreateUserRequest::<String> {
                    params: input.params,
                    query: input.query,
                    headers: input.headers,
                    body: input
                        .body
                        .map(|body| serde_json::to_string(&body))
                        .transpose()?,
                }),
        )
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn create_user(
        &self,
        input: CreateUserRequest,
    ) -> Result<CreateUserResponses, SdkError> {
        let input = CreateUserRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.create_user_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    #[must_use]
    pub fn create_user_raw_request(
        &self,
        input: CreateUserRequest<String>,
    ) -> SdkRequest {
        let path = runtime::path_with_params("/api/users", &input.params);
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    pub async fn create_user_raw(
        &self,
        input: CreateUserRequest<String>,
    ) -> Result<CreateUserResponses, SdkError> {
        let request = self.create_user_raw_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            201 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::CREATE_USER_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status201(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(CreateUserResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Serialize a schema-typed body before constructing the transport request.
    pub fn set_user_role_request(
        &self,
        input: SetUserRoleRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        self.set_user_role_raw_request(SetUserRoleRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input.body.map(|body| serde_json::to_string(&body)).transpose()?,
        })
    }
    /// Execute a schema-typed request. Raw encoded bytes require the explicit `_raw` method.
    pub async fn set_user_role(
        &self,
        input: SetUserRoleRequest,
    ) -> Result<SetUserRoleResponses, SdkError> {
        let input = SetUserRoleRequest::<String> {
            params: input.params,
            query: input.query,
            headers: input.headers,
            body: input
                .body
                .map(|body| serde_json::to_string(&body))
                .transpose()
                .map_err(SdkError::Encode)?,
        };
        self.set_user_role_raw(input).await
    }
    /// Construct a transport request from explicitly encoded wire data.
    pub fn set_user_role_raw_request(
        &self,
        input: SetUserRoleRequest<String>,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/users/:id",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "PATCH",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: input.body,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn set_user_role_raw(
        &self,
        input: SetUserRoleRequest<String>,
    ) -> Result<SetUserRoleResponses, SdkError> {
        let request = self.set_user_role_raw_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::CREATE_USER_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(SetUserRoleResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn disable_user_request(
        &self,
        input: DisableUserRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/users/:id/disable",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn disable_user(
        &self,
        input: DisableUserRequest,
    ) -> Result<DisableUserResponses, SdkError> {
        let request = self.disable_user_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::CREATE_USER_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(DisableUserResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn enable_user_request(
        &self,
        input: EnableUserRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params(
            "/api/users/:id/enable",
            &runtime::request_fields(&input.params)?,
        );
        Ok(SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: input.query,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn enable_user(
        &self,
        input: EnableUserRequest,
    ) -> Result<EnableUserResponses, SdkError> {
        let request = self.enable_user_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::CREATE_USER_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(EnableUserResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn access_review_request(&self, input: AccessReviewRequest) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/access-review",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn access_review(&self) -> Result<AccessReviewResponses, SdkError> {
        self.access_review_with_options(AccessReviewRequest::default()).await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn access_review_with_options(
        &self,
        input: AccessReviewRequest,
    ) -> Result<AccessReviewResponses, SdkError> {
        let request = self.access_review_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::ACCESS_REVIEW_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status200(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(AccessReviewResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    #[must_use]
    pub fn record_access_review_request(
        &self,
        input: RecordAccessReviewRequest,
    ) -> SdkRequest {
        let path = runtime::path_with_params(
            "/api/access-review",
            &::std::collections::BTreeMap::new(),
        );
        SdkRequest {
            method: "POST",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: ::std::collections::BTreeMap::new(),
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        }
    }
    /// Invoke this operation without request parameters or a body.
    pub async fn record_access_review(
        &self,
    ) -> Result<RecordAccessReviewResponses, SdkError> {
        self.record_access_review_with_options(RecordAccessReviewRequest::default())
            .await
    }
    /// Invoke this operation with per-request transport headers.
    pub async fn record_access_review_with_options(
        &self,
        input: RecordAccessReviewRequest,
    ) -> Result<RecordAccessReviewResponses, SdkError> {
        let request = self.record_access_review_request(input);
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            201 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::RECORD_ACCESS_REVIEW_STATUS201_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status201(body))
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(RecordAccessReviewResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
    /// Construct a transport request.
    pub fn list_audit_log_request(
        &self,
        input: ListAuditLogRequest,
    ) -> Result<SdkRequest, serde_json::Error> {
        let path = runtime::path_with_params("/api/audit-log", &input.params);
        Ok(SdkRequest {
            method: "GET",
            url: format!("{}{}", self.runtime.base_url(), path),
            query: runtime::request_fields(&input.query)?,
            headers: input.headers,
            body: None,
            body_bytes: None,
            multipart: None,
            content_type: "application/json",
        })
    }
    pub async fn list_audit_log(
        &self,
        input: ListAuditLogRequest,
    ) -> Result<ListAuditLogResponses, SdkError> {
        let request = self.list_audit_log_request(input).map_err(SdkError::Encode)?;
        let response = self.runtime.execute(request).await?;
        let status = response.status().as_u16();
        let _content_type = runtime::response_content_type(&response)?;
        match status {
            200 if _content_type == "application/json" => {
                let headers = runtime::decode_response_headers::<
                    ListAuditLogResponseHeaders,
                >(response.headers(), schemas::LIST_APPLICATIONS_STATUS200_HEADERS)?;
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_AUDIT_LOG_STATUS200_BODY,
                        None,
                    )
                    .await?;
                Ok(
                    ListAuditLogResponses::Status200(SdkResponseWithHeaders {
                        body,
                        headers,
                    }),
                )
            }
            400 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::LIST_APPLICATIONS_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(ListAuditLogResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
}
