# Migration from the original backend

## Source to target

| Original | Now |
| --- | --- |
| `backend/api/src/main.rs` (router, CSRF guard, startup checks) | `src/api/bootstrapper.rs`, `server.rs`, `application.rs`, `api/transport.rs`, `config.rs` |
| `auth.rs` (cookie extractor, login, logout, audit) | `api/policy.rs`, `api/controllers/auth.rs`, `domain/services/auth.rs`, `persistence/repository.rs` |
| `applications.rs` | `api/controllers/applications.rs`, `domain/services/applications.rs`, `domain/schemas/applications.rs` |
| `documents.rs` (multipart to disk) | `api/controllers/documents.rs`, `domain/services/documents.rs`, `infrastructure/documents.rs` |
| `users.rs` | `api/controllers/users.rs`, `domain/services/users.rs` |
| `crypto.rs` | `domain/crypto.rs` (password hashing, session tokens, legacy SSN cipher) + ForkLaunch `ProtectedField` |
| `seed.rs` | `src/api/seed.rs` |
| `migrations/0001_init.sql` | unchanged, plus `0002_forklaunch.sql` (tenant scope, ciphertext columns, audit trigger) and `0003` to `0014` (one index each, built `CONCURRENTLY`) |
| `backend/worker` (polls `documents`) | `src/extraction` (ForkLaunch leased queue `fl_worker.jobs`) |
| `backend/shared` | `src/shared` |
| `docs` volume | S3-compatible object store |

Users, password hashes, sessions, roles, ids and references are untouched: existing logins keep working.

## Moving an existing database

1. Back up (`docs/runbooks/backup-restore.md`).
2. Restore into the new PostgreSQL, owned by `loandesk_owner`.
3. Run `api --migrate` with `TENANT_ID`, the new `ENCRYPTION_KEY_*`, and
   `LEGACY_DATA_ENCRYPTION_KEY` = the old `DATA_ENCRYPTION_KEY`. It applies `0002`, adopts every row
   into the tenant, re-encrypts SSNs, names, dates of birth, purpose, notes and file names, empties
   the old columns, and fails if any plaintext is left. Rows are read and converted 200 at a time,
   inside one transaction: a failure leaves the database as it was. Indexes are built after that
   schema change, one per migration, without blocking writes; `--migrate` fails if a build was
   interrupted (see `docs/runbooks/failures.md`).
4. Run `extraction --migrate`.
5. Copy each file `<docs volume>/<document id>.pdf` to
   `tenants/<tenant>/applications/<application id>/<document id>.pdf` and update
   `documents.storage_key`. **This step is not scripted yet.** Documents still `pending` in the old
   system also need a job enqueued (`extraction --enqueue`).
6. Verify: `scripts/verify-storage.sh`, row counts per table before and after, sign in, reveal one SSN.

**Rehearsal status:** see the migration report for what was actually rehearsed.

## Rolling back

`0002` drops nothing. Until the old columns are removed in a later migration, rollback is:
restore the pre-migration backup and start the previous release (`docs/runbooks/rollback.md`).
