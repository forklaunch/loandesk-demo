# Architecture

```
browser ──> web (nginx: React app) ──/api/──> api (ForkLaunch, Rust/Axum) ──> PostgreSQL
                │                               │        │                      ▲
                └──/storage/──> object store <──┘        └─ job (same tx) ──┐   │
                     (S3 / MinIO)     ▲                                     ▼   │
                                      └──────────── extraction worker (ForkLaunch leased queue)
```

| Piece | Where | What it does |
| --- | --- | --- |
| Web | `frontend/` | React + TypeScript (Vite), served by nginx. Proxies `/api/` to the API and `/storage/` to the object store. |
| API | `src/api/` | ForkLaunch Rust service. Every route is a typed contract (`api/controllers/*`), bound to the session policy (`api/policy.rs`). |
| Worker | `src/extraction/` | ForkLaunch leased-queue consumer. Reads a PDF from storage, extracts figures, saves them. |
| Shared | `src/shared/` | Risk formulas, figure parser, object-key layout, demo PDF writer. |
| Database | PostgreSQL 16 | One database. Application tables in `public`, the job queue in `fl_worker`. |
| Object store | S3 (MinIO locally) | Uploaded PDFs under `tenants/<tenant>/applications/<application>/<document>.pdf`. |

## Request path

1. `api/transport.rs`: request id, request log, CSRF header check, HSTS/CSP, error message and cookie hand-off.
2. ForkLaunch boundary: exact-origin check, body limit (2 MiB), response deadline (30 s), nosniff / frame / no-store headers, HTTP metrics and trace span.
3. `api/policy.rs`: resolves the session cookie against the database, then enforces the roles declared on the route contract. Denials are logged as `permission_denied`.
4. Controller -> service (`domain/services/*`). Record-level rules live here (own applications only; the creator can never decide).
5. `persistence/repository.rs`: every statement runs in a tenant-bound transaction (row-level security) with a statement timeout. Classified values are sealed before SQL and come back only through an audited reveal. Audit rows are written in the same transaction as the change.

## Documents

Request upload (API records the document as `uploading`, returns a 5-minute POST grant fixed to
one key, `application/pdf` and 20 MB) -> browser posts the file to storage -> complete (API checks
size and the `%PDF-` signature, marks it `pending` and enqueues the job in one transaction) ->
worker -> `done` or `failed`. Downloads use a 60-second signed link, issued after the audit entry.
The API never holds a file: on completion it reads only the first chunk, to check the signature.
The worker reads a file into memory to parse it, and stops at the 20 MB upload limit.

## Data

See `docs/data-classification.md`. Classification is declared on the stored models in
`src/api/persistence/entities/`; the build writes it to `src/api/.forklaunch/compliance.json`.

## Where each control is enforced

| Control | File |
| --- | --- |
| Session lookup, role check, denial logging | `src/api/api/policy.rs` |
| Roles per route | `contract` module of each `src/api/api/controllers/*.rs` |
| Self-approval refused | `domain/services/applications.rs` (`decide`) and the `no_self_decision` constraint |
| Field encryption, audited reveal, tenant transactions | `src/api/persistence/repository.rs` |
| Append-only audit trail, RLS, indexes | `src/api/migrations/0002_forklaunch.sql` |
| Production refusals | `src/api/config.rs` (`Settings::validate`) |
| Upload size and type | `infrastructure/documents.rs`, `domain/services/documents.rs` |
| Idempotent job handling, retry classification | `src/extraction/domain/services/handler.rs` |
| Queue depth and dead-letter metrics | `src/extraction/monitor.rs` |
| What a log line may contain | `src/shared/src/logsafe.rs`, checked by `src/api/__test__/logging.rs` |
| Full SSN released by one audited route only | `ApplicationsService::reveal_ssn`; see `docs/data-classification.md` |
| Readiness (database, object store, not draining), graceful shutdown | `domain/services/health.rs`, `bootstrapper.rs`; worker: `src/extraction/worker.rs` |
| Trace context from request to job | `api/transport.rs` (`traceparent`), `infrastructure/extraction_queue.rs`, `src/extraction/domain/services/handler.rs` (`job_context`) |
| Retry limit and jittered delay | `infrastructure/extraction_queue.rs` (`retry_delays`), `WORKER_MAX_ATTEMPTS`, `WORKER_BACKOFF_SECONDS` |
| Indexes built without blocking writes | `src/api/migrations/0003` to `0014`, checked by `src/api/__test__/persistence.rs` |
| Browser error reports | `frontend/src/errorReporting.ts`, `domain/services/browser_errors.rs` |
| Tests for the above | `src/api/__test__/`, `scripts/e2e.py`, `scripts/verify-storage.sh`, `scripts/verify-operations.sh` |
