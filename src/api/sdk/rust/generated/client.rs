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
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status400(body))
            }
            401 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status401(body))
            }
            403 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status403(body))
            }
            404 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status404(body))
            }
            408 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status408(body))
            }
            409 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status409(body))
            }
            413 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status413(body))
            }
            415 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status415(body))
            }
            429 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status429(body))
            }
            500 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status500(body))
            }
            502 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status502(body))
            }
            503 if _content_type == "application/json" => {
                let body = runtime::decode_json_response(
                        response,
                        schemas::GET_HEALTH_STATUS400_BODY,
                        None,
                    )
                    .await?;
                Ok(GetHealthResponses::Status503(body))
            }
            _ => Err(SdkError::Contract("undeclared response status or content type")),
        }
    }
}
