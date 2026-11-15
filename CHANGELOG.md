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

### Removed
- The original Axum backend (`backend/`) and its Compose file.

## [0.1.0] - 2026-11-15

- LoanDesk as first generated.
