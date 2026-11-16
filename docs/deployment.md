# Deployment

LoanDesk is a ForkLaunch application: `.forklaunch/manifest.toml` describes the `api` service
(PostgreSQL, S3) and the `extraction` worker. The platform builds `Dockerfile.rust` and provides the
database, bucket, secrets, load balancer health checks, log shipping and scaling. The front end
(`frontend/Dockerfile`) is a static site behind the same origin.

A public demo copy with fictional data is a different thing from a deployment: `docs/hosting.md`.

**Not done here:** no deployment has been made and no platform environment exists yet. Everything
below is what a person has to set up.

## Settings

`.env.example` lists every variable. Production requires:

- `APP_ENV=production`, `COOKIE_SECURE=true`, `PUBLIC_ORIGIN=https://...`
- `DATABASE_URL` (role `loandesk_app`) and `MIGRATION_DATABASE_URL` (role `loandesk_owner`) with
  `sslmode=verify-full`; `DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT=false`
- `TENANT_ID`, `ENCRYPTION_KEY_ID`, `ENCRYPTION_KEY_HEX` from the secret manager
- `S3_BUCKET`, `S3_REGION` (task role for credentials; bucket with default encryption and no public access)
- `S3_URL` only for an S3-compatible store, and then `https://` (refused otherwise)
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- the retention and access periods (`RETENTION_*`, `ACCESS_*`, `ENCRYPTION_KEY_MAX_AGE_DAYS`):
  the values in `.env.example` are demo defaults that a person has to confirm (`docs/needs-a-person.md`)
- no `SEED_DEMO_DATA`, no `DEMO_SECRETS`

The worker needs delete permission on the bucket as well as read: its scheduled job removes the
files of purged records.

The API refuses to start in production if any local-demo allowance is set
(`Settings::validate`, tested in `src/api/config.rs`).

## Order of a release

1. Build images tagged with the commit (`GIT_SHA` build arg; shown in the startup log and `/health`).
2. Deploy to staging; run `api --migrate`, then `extraction --migrate`; run `scripts/e2e.py` against staging data that may be thrown away.
   `api --migrate` is safe to run while the previous release is serving: every migration is
   additive, except the ones after `LAST_EXPAND_MIGRATION`, which only drop columns that no
   release since the cutover reads, and only after checking they are empty (`docs/controls.md`, 17).
3. Back up production (`docs/runbooks/backup-restore.md`).
4. Production: run both migrations, then roll the API and worker. Health checks: API `GET /health`
   (readiness) and `GET /api/healthz` (liveness); worker `GET /ready` on `WORKER_HTTP_ADDR`.
   Give both 40 seconds to drain on SIGTERM: from the signal on, the API's readiness probe fails
   (so the load balancer stops sending requests) while in-flight requests finish.
5. Shift traffic gradually if the platform supports it; watch error rate and the `startup` log line for the expected version.

## Health checks, scaling

Run at least two API instances; the API is stateless (sessions are in PostgreSQL, files in S3;
rate-limit buckets are per instance, so a caller can spend the limit once per instance).
Workers can run side by side: jobs are leased, and the hourly housekeeping job runs in one
of them at a time (advisory lock).

Alerts: load `src/monitoring/alerts/loandesk.rules.yaml` into the Mimir and Loki rulers and
`src/monitoring/alerts/alertmanager.yaml` into Alertmanager, with the two destination files it names. Suggested sizing is in `docker-compose.yaml`
(`deploy.resources`).
