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
| Staff email and display name | pii | Declared; **not** field-encrypted: sign-in looks users up by email, the user list is ordered by name, and every application list joins on the name of the relationship manager. Protected by tenant row-level security, admin-only access and storage encryption. |
| Staff account state (role, last sign-in, disabled, password changed) | none | Identifiers and timestamps. |
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

## Retention and deletion

How long each kind of record is kept, and what deletes it. **The periods marked "demo default"
are placeholders so the mechanism runs; they are not a retention policy.** The lending team's
records owner has to confirm or replace each one before real records are held
(`docs/needs-a-person.md`). Each is one setting; changing it needs no code change.

| Data | Kept for | Setting | Deleted by |
| --- | --- | --- | --- |
| Decided applications, with their owners, documents, stored PDFs, extracted figures and search tokens | 2,555 days (7 years) after the decision. **Demo default** | `RETENTION_DECIDED_APPLICATION_DAYS` (0 keeps them) | Worker's scheduled job (`src/extraction/lifecycle.rs`) |
| Applications never decided, with the same | 1,095 days (3 years) after creation. **Demo default** | `RETENTION_UNDECIDED_APPLICATION_DAYS` (0 keeps them) | Same job |
| Upload grants never confirmed, and any file that reached storage | 24 hours. **Demo default** | `RETENTION_UNFINISHED_UPLOAD_HOURS` | Same job (and on the next upload to the same application after 1 hour) |
| Sessions | 8 hours | `SESSION_HOURS` | Expiry checked on every request; expired rows deleted at each sign-in and by the same job. Ended at once by sign-out, password change, role change and disabling |
| Finished extraction jobs (encrypted payload: ids only) | 7 days | `WORKER_COMPLETED_RETENTION_SECONDS` | Worker's queue retention loop |
| Dead-lettered jobs | 30 days | `WORKER_FAILED_RETENTION_SECONDS` | Worker's queue retention loop |
| Staff accounts | While the person is on the team. Unused for 90 days (**demo default**): disabled | `ACCESS_STALE_DAYS` | Disabled, not deleted: applications and audit entries refer to the account. `docs/runbooks/access-review.md` |
| Audit trail | Not deleted by the application. **Period not decided** | none | Nothing: the table refuses `UPDATE`, `DELETE` and `TRUNCATE` (trigger in `0002`). Removing old entries is a deliberate schema-owner operation once a period is decided |
| Backups | **Not decided** | platform | Platform backup retention (`docs/runbooks/backup-restore.md`). A purged record remains in backups until they expire |
| Local container logs | 5 x 10 MB per container | `docker-compose.yaml` | Docker |

### How a purge works

The job runs every hour (`LIFECYCLE_INTERVAL_SECONDS`), in every worker instance, one pass at a
time across all of them (PostgreSQL advisory lock). Per batch of 50 applications
(`LIFECYCLE_BATCH_SIZE`):

1. select the applications past their period, locking them (`FOR UPDATE SKIP LOCKED`);
2. delete the stored PDF of every document of those applications from object storage;
3. in one transaction, delete the applications (owners and document rows go with them through
   `ON DELETE CASCADE`; extracted figures and search tokens are columns of the application row)
   and write one `application_purged` audit entry per application: no user (the job), the
   application id, the time, and `retention=decided days=2555 documents=3`.

If step 3 fails, the rows stay and the next pass repeats the batch; a file is never left behind
without a row that names it. Nothing else holds a copy: there is no cache or search index outside
PostgreSQL, and logs and audit entries hold ids only.

Demonstrated by `retention_periods_are_defined_per_record_type_and_configurable` and
`the_purge_selects_only_records_past_their_period_in_bounded_locked_batches`
(`src/extraction/lifecycle.rs`), and end to end by `scripts/verify-lifecycle.py`, which ages a
decided application past its period and checks that the job removes its rows and its stored
files, writes the audit entry, and leaves a recent application untouched.

### Deleting on request

There is no screen or endpoint to delete one application on request (a borrower asking for
erasure before the period ends). NEEDS A PERSON: whether the lender may do that at all for credit
records is a legal question; until it is answered, the only deletion is by retention period.
