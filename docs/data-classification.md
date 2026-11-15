# Data classification and retention

Classification is code: each stored model in `src/api/persistence/entities/` gives every field
a level, a field without one does not compile, and in a persisted model a classified field can
only be a ForkLaunch `ProtectedField` (ciphertext). `src/api/__test__/persistence.rs` fails if
a migration adds a table or column that no stored model classifies.

| Data | Level | Protection |
| --- | --- | --- |
| Owner name, SSN, date of birth | pii | Field-encrypted (AES-256-GCM, per-tenant derived key, bound to row and purpose). SSN: every response carries the last four only (disclosure projection, the full value is not decrypted); the full value leaves the server only through the audited reveal endpoint (below). |
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

## Full SSNs

The product requirement is "owners (name, SSN, ...)" with a **Show** action next to each owner, so
a full SSN has to be readable by the people working the application. It is released in exactly
one place:

| | |
| --- | --- |
| Route | `POST /api/applications/{id}/owners/{owner_id}/reveal-ssn` (`ApplicationsService::reveal_ssn`) |
| Who | Relationship manager who created the application; credit officers. Never admins (role list on the route contract). Anyone else gets 404, so ids cannot be probed. |
| How much | One owner per call. There is no bulk form; the list, the application view and the export return `ssn_last4` only. |
| Record | `ssn_revealed` and `protected_field_read` audit rows are committed before the value is returned; if they cannot be written, the request fails. |
| Response type | `RevealedSsn`, the only response schema with an `ssn` field. |

Enforced by tests: `a_full_ssn_leaves_the_server_only_through_the_audited_reveal_endpoint` and
`role_rules_match_the_product` (`src/api/__test__/http.rs`), `no_stored_model_keeps_a_readable_ssn`
(`src/api/__test__/persistence.rs`), and `scripts/e2e.py` (creator 200, other manager 404, admin 403,
view and export without the number).

## Logs

Logs never contain the values above. The services log ids, fixed words, counts and booleans:

- Driver and dependency errors are logged through `shared::logsafe::describe`
  (`src/shared/src/logsafe.rs`): for a database error the SQLSTATE code, constraint and table
  name; never the server's message or detail, which can quote a row; never `Debug` output.
- `src/api/__test__/logging.rs` reads every `tracing` call in `src/api`, `src/extraction` and
  `src/shared` and fails on one that uses `Debug`, is computed from a request body or a
  personal or confidential field, logs an error another way, or interpolates into its message.
- Browser error reports are reduced to a class name, a route without ids and stack frames
  before they are logged (`src/api/domain/services/browser_errors.rs`); the message is not accepted.
- The `traceparent` request header is accepted only when well-formed (`api/transport.rs`).
- nginx logs request paths without query strings (`frontend/nginx.conf`): signed storage links
  carry a signature and the file name there.
- `scripts/verify-operations.sh` searches every container's log for the demo owners' names,
  SSNs, dates of birth, free text, file names, passwords and link signatures after an end-to-end run.

## Retention

| Data | Period | Enforced by |
| --- | --- | --- |
| Sessions | 8 hours (`SESSION_HOURS`) | Expiry checked on every request; expired rows deleted at each sign-in |
| Unused upload grants | 1 hour | `DocumentsService::request_upload` |
| Finished extraction jobs | 7 days (`WORKER_COMPLETED_RETENTION_SECONDS`) | Worker retention loop |
| Dead-lettered jobs | 30 days (`WORKER_FAILED_RETENTION_SECONDS`) | Worker retention loop |
| Local container logs | 5 x 10 MB per container | `docker-compose.yaml` logging options |
| Applications, owners, documents, audit trail | **Not decided** | Not implemented: needs the lending team's record-retention policy (see NEEDS A PERSON in the migration report) |
