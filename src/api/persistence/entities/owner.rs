//! A business owner (a person). Everything that identifies the person is ciphertext.
use forklaunch_core::persistence::{compliance::ComplianceMetadata, protected::ProtectedField};
use uuid::Uuid;

/// Plaintext cannot be assigned to a classified persisted field.
/// ```compile_fail
/// use api::persistence::entities::StoredOwner;
/// let row = StoredOwner {
///     id: uuid::Uuid::nil(), tenant_id: "tenant".into(), application_id: uuid::Uuid::nil(),
///     full_name_ciphertext: String::from("Eleanor Whitfield"),
///     ssn_ciphertext: String::from("900114321"),
///     date_of_birth_ciphertext: String::from("1978-04-12"),
///     ownership_percent: 60.0,
/// };
/// ```
#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "Owner", persisted)]
pub struct StoredOwner {
    #[compliance(level = "none")]
    pub id: Uuid,
    #[compliance(level = "none")]
    pub tenant_id: String,
    #[compliance(level = "none")]
    pub application_id: Uuid,
    #[compliance(level = "pii", purpose = "owner-full-name-v1")]
    pub full_name_ciphertext: ProtectedField,
    /// Sealed with the last-four disclosure policy: screens get the last four
    /// digits without the full number ever being decrypted for them.
    #[compliance(level = "pii", purpose = "owner-ssn-v1")]
    pub ssn_ciphertext: ProtectedField,
    #[compliance(level = "pii", purpose = "owner-date-of-birth-v1")]
    pub date_of_birth_ciphertext: ProtectedField,
    #[compliance(level = "none")]
    pub ownership_percent: f64,
}
