#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct LoginRequest {
    #[field(max_length = 254)]
    pub email: String,
    #[field(max_length = 1024)]
    pub password: String,
    /// Six-digit authenticator code; required once two-factor sign-in is enabled.
    #[field(optional, max_length = 12)]
    pub totp: forklaunch_core::contract::schema::Optional<String>,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub mfa_enabled: bool,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ok {
    pub ok: bool,
}

/// Returned once, when enrolment starts. The secret is stored encrypted and
/// only becomes active after `MfaCode` confirms the authenticator works.
#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct MfaEnrolment {
    pub secret: String,
    pub otpauth_url: String,
}

#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct MfaCode {
    #[field(max_length = 12)]
    pub code: String,
}
