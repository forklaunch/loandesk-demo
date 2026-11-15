# Deployment

LoanDesk is a ForkLaunch application: `.forklaunch/manifest.toml` describes the `api` service
(PostgreSQL, S3) and the `extraction` worker. The platform builds `Dockerfile.rust` and provides the
database, bucket, secrets, load balancer health checks, log shipping and scaling. The front end
(`frontend/Dockerfile`) is a static site behind the same origin.

**Not done here:** no deployment has been made and no platform environment exists yet. Everything
below is what a person has to set up.

## Settings

`.env.example` lists every variable. Production requires:

- `APP_ENV=production`, `COOKIE_SECURE=true`, `PUBLIC_ORIGIN=https://...`
- `DATABASE_URL` (role `loandesk_app`) and `MIGRATION_DATABASE_URL` (role `loandesk_owner`) with
  `sslmode=verify-full`; `DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT=false`
- `TENANT_ID`, `ENCRYPTION_KEY_ID`, `ENCRYPTION_KEY_HEX` from the secret manager
- `S3_BUCKET`, `S3_REGION` (task role for credentials; bucket with default encryption and no public access)
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- no `SEED_DEMO_DATA`, no `DEMO_SECRETS`

The API refuses to start in production if any local-demo allowance is set
(`Settings::validate`, tested in `src/api/config.rs`).

## Order of a release

1. Build images tagged with the commit (`GIT_SHA` build arg; shown in the startup log and `/health`).
2. Deploy to staging; run `api --migrate`, then `extraction --migrate`; run `scripts/e2e.py` against staging data that may be thrown away.
3. Back up production (`docs/runbooks/backup-restore.md`).
4. Production: run both migrations, then roll the API and worker. Health checks: API `GET /health`
   (readiness) and `GET /api/healthz` (liveness); worker `GET /ready` on `WORKER_HTTP_ADDR`.
   Give both 40 seconds to drain on SIGTERM.
5. Shift traffic gradually if the platform supports it; watch error rate and the `startup` log line for the expected version.

## Health checks, scaling

Run at least two API instances; the API is stateless (sessions are in PostgreSQL, files in S3).
Workers can run side by side: jobs are leased. Suggested sizing is in `docker-compose.yaml`
(`deploy.resources`).
