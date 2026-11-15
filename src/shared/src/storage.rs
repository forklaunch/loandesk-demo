//! Where document files live in object storage. The API (writer) and the
//! worker (reader) must agree, so the layout is defined once, here.

use std::fmt::Display;

/// Largest PDF the app accepts. Enforced three times: by the upload form, by
/// the storage upload grant (object storage itself rejects larger bodies), and
/// when the upload is confirmed.
pub const MAX_UPLOAD_BYTES: u64 = 20 * 1024 * 1024;

/// At most this many documents per application.
pub const MAX_DOCUMENTS_PER_APPLICATION: i64 = 50;

/// Every file sits under its tenant's prefix, then its application's.
pub fn tenant_prefix(tenant: impl Display) -> String {
    format!("tenants/{tenant}/")
}

pub fn document_key(
    tenant: impl Display,
    application: impl Display,
    document: impl Display,
) -> String {
    format!("{}applications/{application}/{document}.pdf", tenant_prefix(tenant))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_keys_are_under_the_tenant_prefix() {
        let key = document_key("t1", "a1", "d1");
        assert_eq!(key, "tenants/t1/applications/a1/d1.pdf");
        assert!(key.starts_with(&tenant_prefix("t1")));
        assert!(!key.starts_with(&tenant_prefix("t2")));
    }
}
