# Changelog

All notable changes to LoanDesk. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Every pull request adds a line under "Unreleased"; a release moves them under a version and date.

## [Unreleased]

### Changed
- The manifest projects are `loans-service` (the API) and `loans` (the worker), and the Compose
  services `loans-service` and `loans-worker`, so the platform gives both one database. Crates and
  binaries keep their names. `extraction --migrate` now applies the API's migrations too, and the
  API waits for the schema at startup (`SCHEMA_WAIT_SECONDS`). Shared local settings moved from the
  Compose file to `compose.backend.env`.
- `DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT` and `WORKER_ALLOW_PLAINTEXT_DEVELOPMENT` may be left unset,
  which means `false`. The image sets `RUST_LOG=info` unless told otherwise.
- The backend runs on ForkLaunch: one Rust API service (`src/api`) with typed route contracts,
  and one Rust worker (`src/extraction`) on the ForkLaunch leased PostgreSQL queue.
- Documents are uploaded straight to object storage with a short-lived, size-capped grant and
  downloaded through a one-minute signed link. `POST /api/applications/{id}/documents` now takes
  JSON and returns the grant; `POST /api/documents/{id}/complete` confirms the upload;
  `POST /api/documents/{id}/download-url` replaces `GET /api/documents/{id}/download`.
- Search moved from `GET /api/applications?q=` to `POST /api/applications/search` so EINs and
  names never appear in URLs.
- Lists return one page (50 by default, 100 at most) and an `x-next-cursor` header.
- Owner names, SSNs, dates of birth, purpose, decision notes and file names are stored as
  ciphertext. Search on owner names and purpose matches whole words or word prefixes of three
  or more letters (previously any substring).
- A role change ends that user's sessions.
- Secrets for the local stack are generated on first start; none are in the repository.

### Added
- Hosted demo mode (`HOSTED_DEMO=true`, `docs/hosting.md`): the API serves the front end itself
  from `src/api/web-dist`, carries document files (`PUT`/`GET /api/documents/{id}/content`, 4 MB at
  most), requires HTTPS, `Secure` cookies, a verified-TLS database connection and a demo password
  other than the public one, and the front end shows a demo banner.
- `GET /api/client-settings` (public): demo banner, upload route and size limit for the front end.
- Two-factor sign-in (TOTP) under "Security"; required for admins when `APP_ENV=production`.
- `Idempotency-Key` on application creation.
- `GET /api/applications/{id}/export`.
- Liveness (`/api/healthz`) and readiness (`/api/readyz`, `/health`) probes.
- Structured JSON logs with request ids, OTLP export, job and queue metrics, alert rules.
- CI checks, CODEOWNERS, pull request template, runbooks, architecture and decision records.

### Security
- Errors from the database driver and other dependencies are logged as an SQLSTATE code and
  constraint name, never the driver's text or `Debug` output, which can quote a row
  (`shared::logsafe`). A test reads every log statement in the source.
- The web server's access log no longer records query strings (signed storage links carry a
  signature and the file name there). A malformed `traceparent` header is ignored.
- Dependencies with known vulnerabilities upgraded: vite 5.4.11 to 6.4.6 (and esbuild with it),
  multer 2.4.0 to 2.5.0 and @opentelemetry/core 1.30.1 to 2.12.1 in the generated workspace
  libraries, pdf-extract 0.7 to 0.12 (lopdf 0.34 to 0.42) in the worker.
- The README and `docs/data-classification.md` state exactly how a full SSN can leave the server:
  one audited, role-limited route. Behaviour is unchanged.

### Added (operations)
- `POST /api/client-errors`: the front end reports uncaught errors, rejected promises and
  rendering failures (class and stack frames only); they are logged as `browser_error`.
- Error-budget burn alerts for the availability objective, and an alert on browser errors.
- `scripts/verify-operations.sh`, and CI audits of every lockfile.

### Changed (operations)
- Indexes are built with `CREATE INDEX CONCURRENTLY`, one per migration (`0003` to `0014`), and
  no longer inside `0002`. **`0002_forklaunch.sql` was edited**: it had not been applied to any
  deployed database. A local Compose volume created before this change must be recreated
  (`docker compose down -v`).
- The one-time re-encryption of legacy rows reads 200 rows at a time.
- The API's readiness probe fails from the moment shutdown starts, so no new requests arrive while
  in-flight ones finish.
- A queued job carries the trace context of the request's own server span, and the worker's span
  for the job is a child of it. Each job's retry delay is drawn between `WORKER_BACKOFF_SECONDS`
  and twice that.
- Every request the front end makes has a deadline.

### Added (records, access and keys)
- **Retention purge.** The worker runs a housekeeping pass every hour: applications past their
  retention period are deleted with their owners, documents and stored PDFs, and each leaves an
  `application_purged` audit entry. Periods are settings (`RETENTION_*`); the shipped values are
  demo defaults to be confirmed (`docs/data-classification.md`). Upload grants never confirmed are
  removed with their files. One pass runs at a time across all workers. `extraction --lifecycle-once`
  runs a pass by hand.
- **Access review.** `GET /api/access-review` (and the Users page) lists every account with role,
  second factor, last sign-in, open sessions and whether it is stale; `POST /api/access-review`
  records a review in the audit trail. Admins can disable and re-enable accounts
  (`POST /api/users/{id}/disable`, `/enable`). Accounts unused for `ACCESS_STALE_DAYS` are disabled
  by the same hourly pass; the last usable admin never is.
- **Password change**: `POST /api/auth/password` and a form under Security. It ends every session
  of the user and starts a new one for the caller.
- **Key rotation**: `api --rotate-keys` re-encrypts every protected field under the active key and
  rebuilds the search tokens; search matches rows indexed under either key while it runs.
  `docs/runbooks/key-rotation.md`. The worker warns when the active key is older than
  `ENCRYPTION_KEY_MAX_AGE_DAYS`.
- **Audit trail for admins**: `GET /api/audit-log`, paginated.
- **Rate limits**: sign-in per client address, requests and expensive operations per user (HTTP 429).
- Alert rules for security events, the scheduled job, the access review and key rotation reminders,
  panics and dependency calls; Alertmanager routing by severity; `docs/runbooks/security-events.md`.
- Metrics for calls to PostgreSQL and the object store (`loandesk.dependency.*`).
- `docs/controls.md` (every control, with the test that proves it), `docs/needs-a-person.md`,
  `docs/data-sharing.md`; `scripts/verify-lifecycle.py`, `scripts/load-test.py`; a coverage job and a
  branch-protection ruleset (`.github/rulesets/main.json`, applied with `scripts/apply-branch-protection.sh`).

### Changed (records, access and keys)
- **Requests with a field the API does not declare are refused (400)**; they used to be ignored.
- **Migration `0019` drops the pre-encryption plaintext columns** (`owners.full_name`,
  `ssn_encrypted`, `ssn_last4`, `date_of_birth`; `applications.purpose`, `decision_note`;
  `documents.filename`). `api --migrate` applies the additive migrations, re-encrypts, checks that
  nothing readable is left, and only then drops them. After it, going back to the pre-ForkLaunch
  backend means restoring the backup taken before the cutover.
- The one-time re-encryption commits each batch of 200 rows on its own and resumes after an interruption.
- A disabled account cannot sign in and has no sessions; the answer is the same as for a wrong password.
- `User` responses carry `last_login_at` and `disabled_at`.
- Dependency failures are logged with the code location of the failing call.
- In production an `S3_URL` that is not `https://` is refused, by the API and the worker.
- Base images are pinned to full versions: `debian:12.13-slim`, `node:22.21.1-alpine3.22`,
  `nginxinc/nginx-unprivileged:1.27.5-alpine3.21`, `memcached:1.6.39-alpine3.22`.

### Removed
- The original Axum backend (`backend/`) and its Compose file.

## [0.1.0] - 2026-11-15

- LoanDesk as first generated.
