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
| `migrations/0001_init.sql` | unchanged, plus `0002_forklaunch.sql` (tenant scope, ciphertext columns, audit trigger), `0003` to `0014` and `0016` to `0018` (one index each, built `CONCURRENTLY`), `0015` (account columns) and `0019` (drops the emptied plaintext columns) |
| `backend/worker` (polls `documents`) | `src/extraction` (ForkLaunch leased queue `fl_worker.jobs`) |
| `backend/shared` | `src/shared` |
| `docs` volume | S3-compatible object store |

Users, password hashes, sessions, roles, ids and references are untouched: existing logins keep working.

## Moving an existing database

1. Back up (`docs/runbooks/backup-restore.md`).
2. Restore into the new PostgreSQL, owned by `loandesk_owner`.
3. Run `api --migrate` with `TENANT_ID`, the new `ENCRYPTION_KEY_*`, and
   `LEGACY_DATA_ENCRYPTION_KEY` = the old `DATA_ENCRYPTION_KEY`. In this order it:
   - applies the additive migrations (`0002` to `0018`): `0002` adopts every row into the tenant
     and adds the ciphertext columns; indexes are built one per migration without blocking writes;
   - re-encrypts SSNs, names, dates of birth, purpose, notes and file names and empties the old
     columns, 200 rows at a time, each batch committed on its own, so an interrupted run
     continues where it stopped when `--migrate` is run again;
   - fails if any plaintext is left; and only then
   - applies `0019`, which drops the old columns (and itself refuses to run on readable data).

   It also fails if an index build was interrupted (see `docs/runbooks/failures.md`).
   `0002` updates each table in one statement: run this step on the restored database before any
   service is started against it, which is the order given here.
4. Run `extraction --migrate`.
5. Copy each file `<docs volume>/<document id>.pdf` to
   `tenants/<tenant>/applications/<application id>/<document id>.pdf` and update
   `documents.storage_key`. **This step is not scripted yet.** Documents still `pending` in the old
   system also need a job enqueued (`extraction --enqueue`).
6. Verify: `scripts/verify-storage.sh`, row counts per table before and after, sign in, reveal one SSN.

**Rehearsal status (2026-11-15).** Step 3 was rehearsed on the local stack against a database
built from `0001_init.sql` alone and filled in the old format: 450 applications with plaintext
purpose and decision notes, 450 owners with plaintext names and dates of birth and SSNs under the
old backend's own AES-256-GCM key, 90 documents with plaintext file names. `api --migrate`
converted them in three batches, reported `legacy_rows_before=990 legacy_rows_after=0`, and then
applied `0019`; afterwards `api --rotate-keys` under a second key decrypted and re-sealed every one
of those values, which shows each was converted to valid ciphertext. Not rehearsed: a restore from
a real backup of the old system (none exists), step 5 (moving the files), and an interrupted run.

## Rolling back

Rollback to the old backend is: restore the pre-migration backup and start the previous release
(`docs/runbooks/rollback.md`). The migrated database cannot be handed back to it: its plaintext
columns are emptied by the re-encryption and dropped by `0019`.
