# Data classification and retention

Classification is code: each stored model in `src/api/persistence/entities/` gives every field
a level, a field without one does not compile, and in a persisted model a classified field can
only be a ForkLaunch `ProtectedField` (ciphertext). `src/api/__test__/persistence.rs` fails if
a migration adds a table or column that no stored model classifies.

| Data | Level | Protection |
| --- | --- | --- |
| Owner name, SSN, date of birth | pii | Field-encrypted (AES-256-GCM, per-tenant derived key, bound to row and purpose). SSN: last four via disclosure projection; full value only through the audited reveal endpoint. |
| Application purpose, decision note | pii (free text) | Field-encrypted. Searchable through keyed word-prefix tokens. |
| Uploaded file name | pii (free text) | Field-encrypted. |
| Uploaded PDF | pii | Object storage, tenant prefix, encrypted at rest (SSE), reachable only through short-lived signed grants. `documents.classification` records the level. |
| Two-factor secret | pii (credential) | Field-encrypted. |
| Staff email and display name | pii | Declared; **not** field-encrypted (sign-in looks up by email, lists join on name). Tenant isolation, admin-only access, storage encryption. |
| Business legal name, EIN, address, industry, amounts, extracted figures | none (confidential business data) | Not personal data. Restricted by role and tenant; storage encryption. The framework has no "confidential" level, so this is recorded here. |
| Password | n/a | Argon2id hash only. |
| Session token | n/a | SHA-256 only. |
| Audit trail | none | Identifiers and fixed action names only. Append-only (trigger). |
| Queued job payload | pii | Encrypted before the queue stores it. |

Logs never contain these values: services log ids, booleans and fixed event names.

## Retention

| Data | Period | Enforced by |
| --- | --- | --- |
| Sessions | 8 hours (`SESSION_HOURS`) | Expiry checked on every request; expired rows deleted at each sign-in |
| Unused upload grants | 1 hour | `DocumentsService::request_upload` |
| Finished extraction jobs | 7 days (`WORKER_COMPLETED_RETENTION_SECONDS`) | Worker retention loop |
| Dead-lettered jobs | 30 days (`WORKER_FAILED_RETENTION_SECONDS`) | Worker retention loop |
| Local container logs | 5 x 10 MB per container | `docker-compose.yaml` logging options |
| Applications, owners, documents, audit trail | **Not decided** | Not implemented: needs the lending team's record-retention policy (see NEEDS A PERSON in the migration report) |
