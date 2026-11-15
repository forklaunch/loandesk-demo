# Changelog

All notable changes to LoanDesk. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Every pull request adds a line under "Unreleased"; a release moves them under a version and date.

## [Unreleased]

### Changed
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

### Removed
- The original Axum backend (`backend/`) and its Compose file.

## [0.1.0] - 2026-11-15

- LoanDesk as first generated.
