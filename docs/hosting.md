# Hosting the demo on ForkLaunch

How the public demo copy of LoanDesk is meant to run on the ForkLaunch platform, which settings
each part needs, and what is still unproven. **No deployment has been made from this document.**
Everything marked *verified* was read from code or run locally (the last section says which);
everything else is listed under "Not verified".

The platform facts below were read from `forklaunch-platform` at `origin/main` `238bbf9f4`
(2026-11-16), mostly `src/modules/deployment-agent-worker/domain/services/pulumi-generator.service.ts`,
`utils/native-runtime.util.ts`, `utils/object-store.util.ts` and `utils/microvm-compute.ts`.

## What runs where

| Part | Compute | Why |
| --- | --- | --- |
| `api` (`/app/bin/api`) | Scale to zero: a Lambda MicroVM behind the platform's wake router, one public HTTPS hostname | It only works when someone is looking at the demo. It holds no state between requests (sessions are in PostgreSQL, files in the bucket), so it can be suspended and woken. It serves the front end itself, so there is nothing else to host. |
| `extraction` (`/app/bin/extraction`) | Always on, shared ECS pool | It polls the PostgreSQL queue and runs the hourly housekeeping job; nothing calls it, so nothing would wake it. The platform runs workers on ECS only. |
| PostgreSQL | Managed | |
| Documents | Managed S3 bucket | |

There is no nginx. In a hosted demo the API:

- answers every `GET` outside `/api`, `/health`, `/mcp` and `/docs` from the front end compiled
  into the binary (`src/api/web-dist`, `src/api/api/web.rs`), with the security headers nginx sets
  for Compose (same Content-Security-Policy, `nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and HSTS because cookies are `Secure`);
- carries document files itself: `PUT /api/documents/{id}/content` to upload,
  `GET /api/documents/{id}/content` to download. Uploads are capped at 4 MB there (20 MB in
  Compose), still PDF only, still checked again when the upload is completed;
- serves no OpenAPI UI, and ignores `CORS_ORIGINS`.

### Why files go through the API

The platform's managed bucket does allow browser uploads from the app's own hostnames (a CORS rule
for `GET, PUT, POST, HEAD`, on by default: `services/templates/object-store-s3.pulumi.ts`), so
signed direct uploads would have been possible in principle. They were not chosen because:

- the same code path has to work with local MinIO, which a browser cannot reach without the
  `/storage/` proxy that only the nginx container provides;
- the page's Content-Security-Policy would have to name the bucket's host, which is only known at
  run time;
- the API cannot sign anything for that bucket today (next section).

Through the API there is one path for both stores, no CORS rule and no policy exception to get
wrong. The cost is the 4 MB cap, which the demo's PDFs (about 1 kB each) are far below.

## The front end in the image

The platform builds `Dockerfile.rust`, a Rust toolchain with no Node. So the built front end is
committed, as sheet-env does it: `src/api/web-dist` is the output of

```sh
cd frontend && npm ci && npm run build:embed
```

and `include_dir!` compiles it into the `api` binary. After any change under `frontend/`, run that
command and commit the result; the "Front end" job in `.github/workflows/checks.yml` rebuilds it
and fails if the committed copy differs. (The build was compared between macOS/Node 24 and
`node:22.21.1-alpine3.22`: identical files.)

## Settings

`HOSTED_DEMO=true` is the one switch. With it the API refuses to start unless the public origin is
`https://`, cookies are `Secure` (the default with the flag), `DEMO_SECRETS` is off, the database
connection is not plaintext, `S3_URL` (if any) is `https://`, and `DEMO_PASSWORD` is set and is not
the password printed in this repository's README. It also refuses `APP_ENV=production`: a hosted
demo seeds demo logins, which production never does. The worker refuses
`WORKER_ALLOW_PLAINTEXT_DEVELOPMENT=true` when the flag is on. Tests: `src/api/config.rs`,
`src/extraction/domain/schemas/events.rs`.

Do **not** set `APP_ENV=production`, `DEMO_SECRETS`, `S3_URL`, `S3_PUBLIC_URL`, `COOKIE_SECURE=false`
or either `*_ALLOW_PLAINTEXT_DEVELOPMENT=true`.

Example values are placeholders. Never commit or paste a real one.

### `api` (scale to zero)

| Variable | Set by | Example | Notes |
| --- | --- | --- | --- |
| `DATABASE_URL` | Platform | `postgresql://api_app:…@host:5432/api_database?sslmode=verify-full&sslrootcert=/app/certs/rds-global-bundle.pem` | The restricted runtime role (`NOBYPASSRLS`, DML only). The framework refuses a role that is superuser, can bypass row-level security or can create roles. |
| `DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT` | Platform | `false` | Optional for LoanDesk; unset means `false`. |
| `PUBLIC_ORIGIN` | Platform | `https://api.<stack>.<base domain>` | Set for Rust services when the application has DNS. Must be exactly the address browsers use: the API refuses requests whose `Origin` differs. |
| `NODE_ENV`, `OTEL_*`, `*_URL`, `WS_PORT`, `DOCS_SERVER_URLS`, `DOCS_SERVER_DESCRIPTIONS`, `MCP_ALLOWED_HOSTS`, `CORS_ORIGINS` | Platform | | Injected for every Rust service; LoanDesk reads only the `OTEL_*` ones. About 20 variables. |
| `HOST`, `PORT`, `PROTOCOL`, `VERSION`, `DOCS_PATH`, `RUST_LOG` | Image | `0.0.0.0`, `8000`, `http`, `v1`, `/docs`, `info` | Defaults in `Dockerfile.rust`. The platform does not set them and routes to port 8000 when `PORT` is not stored. `RUST_LOG` is required at startup. |
| `HOSTED_DEMO` | Us | `true` | |
| `SEED_DEMO_DATA` | Us | `true` | Creates the demo logins and applications on an empty database. |
| `DEMO_PASSWORD` | Us (secret) | `<12+ characters, not the README's>` | Password of every seeded demo login. |
| `TENANT_ID` | Us | `<uuid>` | Same value on both parts. |
| `STORAGE_ENCRYPTION_NAMESPACE` | Us | `loandesk-records` | Same value on both parts. |
| `ENCRYPTION_KEY_ID` | Us | `primary` | Same value on both parts. |
| `ENCRYPTION_KEY_HEX` | Us (secret) | `<64 hex digits: openssl rand -hex 32>` | Same value on both parts: the worker opens what the API seals. |
| `ENCRYPTION_LEGACY_KEY_HEX` | Us | `disabled` | Same value on both parts. |
| `S3_BUCKET`, `S3_REGION` | Us, for now | `<bucket>`, `us-west-2` | See "Bucket access from the API". |
| `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | Us (secret), for now | | See "Bucket access from the API". |
| `WORKER_QUEUE` | Us | `extraction` | The API enqueues extraction jobs. Same value on both parts. |
| `WORKER_MAX_ATTEMPTS`, `WORKER_BACKOFF_SECONDS` | Us | `3`, `5` | Same values on both parts. |

That is about 37 variables, under the 50 a MicroVM image can carry before the platform switches to
delivering them in a file (which needs `/bin/sh`; the image has one). A MicroVM's environment is
baked into its image at build time, so changing any value means a new deploy.

Optional, with defaults: `SESSION_HOURS` (8), `MFA_REQUIRED_FOR_ADMINS` (off), `RATE_LIMIT_*`
(60 / 600 / 60 per minute), `DATABASE_MAX_CONNECTIONS` (10), `DATABASE_STATEMENT_TIMEOUT_MS`
(10000), `ACCESS_STALE_DAYS` and `ACCESS_REVIEW_INTERVAL_DAYS` (90).

### `extraction` (always on)

| Variable | Set by | Example | Notes |
| --- | --- | --- | --- |
| `DATABASE_URL` | Platform | as above | **Must be the API's database.** See "One database for two parts". |
| `S3_BUCKET`, `S3_REGION` | Platform | | When the worker is a consumer of the managed bucket; credentials come from the ECS task role. The worker reads files and, in the retention job, deletes them. |
| `NODE_ENV`, `OTEL_*`, `*_URL`, `WS_PORT`, `DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT` | Platform | | |
| `RUST_LOG` | Image | `info` | |
| `HOSTED_DEMO` | Us | `true` | Makes the worker refuse a plaintext allowance. |
| `TENANT_ID`, `STORAGE_ENCRYPTION_NAMESPACE`, `ENCRYPTION_KEY_ID`, `ENCRYPTION_KEY_HEX`, `ENCRYPTION_LEGACY_KEY_HEX` | Us | | The same values as the API. |
| `WORKER_QUEUE`, `WORKER_MAX_ATTEMPTS`, `WORKER_BACKOFF_SECONDS` | Us | `extraction`, `3`, `5` | The same values as the API. |
| `WORKER_CONCURRENCY`, `WORKER_BATCH_SIZE` | Us | `2`, `1` | Required. |
| `WORKER_POLL_MS`, `WORKER_LEASE_MS`, `WORKER_RENEW_MS`, `WORKER_DRAIN_MS` | Us | `500`, `60000`, `20000`, `30000` | Required. |
| `WORKER_RETENTION_INTERVAL_MS`, `WORKER_RETENTION_BATCH_LIMIT`, `WORKER_RETENTION_TIMEOUT_MS` | Us | `60000`, `100`, `5000` | Required. |

Optional: `WORKER_COMPLETED_RETENTION_SECONDS`, `WORKER_FAILED_RETENTION_SECONDS`,
`WORKER_HTTP_ADDR` with `WORKER_CONTROL_TOKEN` (the platform puts no health check on a worker),
`RETENTION_*`, `LIFECYCLE_*`, `ACCESS_*`, `ENCRYPTION_KEY_MAX_AGE_DAYS` (`.env.example` has the defaults).

The platform can mint `ENCRYPTION_KEY_HEX`, `TENANT_ID`, `STORAGE_ENCRYPTION_NAMESPACE` and
`ENCRYPTION_KEY_ID` itself, but per component (the namespace would be `<component>-records`), and
only when the release manifest lists them as required. LoanDesk needs **one** value of each shared
by both parts, so store them at application scope by hand before the first deploy.

## Migrations

For a scale-to-zero Rust service the platform runs the migration as a one-off task before it builds
the MicroVM image (platform PR #1524): the same image, command `/app/bin/<binary> --migrate`, with
the schema-owner URL in both `DATABASE_URL` and `MIGRATION_DATABASE_URL`, and the deploy fails if it
exits non-zero. For an always-on component the same command is an init container of every task.

Both binaries fit that as they are:

- `/app/bin/api --migrate` reads `MIGRATION_DATABASE_URL`, plus `TENANT_ID` and the `ENCRYPTION_*`
  settings (it registers the active key);
- `/app/bin/extraction --migrate` reads `DATABASE_URL` (the owner URL in that task) and creates the
  queue's tables.

Order matters once: the API seeds on first start and enqueues jobs, so the queue's tables must
exist first. Locally the order is `api --migrate`, `extraction --migrate`, then both processes.
On the platform the two migrations belong to different components and nothing orders the worker's
before the API's first start (see "Not verified").

## Known gaps on the platform side

These were found by reading the platform's generator. Each one stops a first deploy from working
until it is settled; none can be fixed from this repository alone.

### Bucket access from the API

A scale-to-zero service gets `S3_BUCKET` and `S3_REGION`, but its MicroVM role may only write its
log (`microvm-compute.ts`, "The running MicroVM's own role: it only writes its log"); the bucket
grant goes to the ECS task role only, and the platform strips `S3_ACCESS_KEY_ID` /
`S3_SECRET_ACCESS_KEY` from a component that uses a platform bucket. So with nothing else done, the
API cannot reach the bucket: seeding fails at first start, and `/health` (which checks the bucket)
reports not ready.

Until the platform grants the MicroVM role the bucket: create an IAM user limited to the demo's
bucket (`s3:GetObject`, `s3:PutObject`, `s3:PutObjectTagging`, `s3:DeleteObject`, `s3:ListBucket`)
and store its key as `S3_ACCESS_KEY_ID` / `S3_SECRET_ACCESS_KEY` on the `api` component. A
component with a stored key "brings its own storage": the platform then leaves its `S3_*` alone, so
`S3_BUCKET` and `S3_REGION` have to be stored for it too. LoanDesk reads static keys without
`S3_URL` (the framework's `S3Environment`), which is this case.

### One database for two parts

The platform gives every component its own database (`<component>_database`) and its own pair of
roles (`<module>_owner`, `<module>_app`), where a module is a service and the worker of the same
name (`notes-service` + `notes-worker`). `api` and `extraction` are two modules to it. LoanDesk's
worker is not a separate module in that sense: it takes jobs from, and writes figures to, the API's
database (`documents`, `applications`, the queue tables).

So as generated, the worker would get an empty database of its own and process nothing. Storing
the same `DB_NAME` on both does not solve it by itself: each module's provisioning task makes its
own owner role the owner of the database and of schema `public`, and the worker's app role gets no
rights on tables the API's owner created.

Options, for whoever owns the platform side: let a component bind to another component's database
and runtime role; or rename the worker so the platform sees one module (`api` + `api-worker`),
which is a rename across this repository; or run the worker on a hand-stored `DATABASE_URL` for the
API's runtime role, if the platform keeps a stored value for a component with no database resource.

### The worker and the bucket

`.forklaunch/manifest.toml` declares `object_store = "s3"` for `api` only. The worker reads and
deletes files, so it must be a consumer of the same bucket to get `S3_BUCKET`, `S3_REGION` and the
task-role grant. How the CLI turns per-project resources into the release manifest's resource list
(one bucket per resource name) was not checked; the manifest was left unchanged.

## Not verified

- Anything on the platform itself: no release, deployment or environment variable was created.
- That the CLI's release manifest for this repository marks `api` and `extraction` as `runtime:
  tokio` with a `migrate` entry (`binary`, default args `--migrate`). Without a `migrate` entry the
  platform runs no migration.
- The request limit through the gateway. The wake router's code says 6 MB
  (`src/modules/wake-router/src/main.rs`); the figure given for this work was about 4.5 MB. The
  upload cap is 4 MiB (4,194,304 bytes) of raw PDF, under both, **provided** the gateway passes a
  binary body through as it is. If it base64-encodes bodies on the way to the MicroVM, 4 MiB becomes
  5.6 MB and the cap has to come down to about 3 MB (`HOSTED_DEMO_MAX_UPLOAD_BYTES` in
  `src/api/config.rs`).
- That the worker's migration has run before the API first starts. If it has not, the API's first
  start fails while seeding and succeeds on a later wake.
- That `PUBLIC_ORIGIN` is the hostname people will use. If the demo gets a custom domain, the value
  must be that domain or every state-changing request is refused with "Origin denied".
- Cold start. On a wake the API connects to PostgreSQL and the bucket before it listens; the wake
  router retries for 20 seconds. Not measured for this image.
- Rate limits and the sign-in limit are kept in the memory of one instance and start again after a
  suspend or wake. The limit that must hold (five wrong passwords per account) is in PostgreSQL.
- The framework answers within 30 seconds or gives up; a 4 MB upload on a slow connection can hit
  that.
- "This copy resets periodically": nothing in this repository resets it. Seeding runs only on an
  empty `users` table, so a reset is: empty the database and the bucket, then let the API start.
  Who does that, and how often, is undecided.
- The `MCP` endpoint the framework mounts at `/mcp` is reachable on the public hostname as it is on
  the API port in Compose. Its tools call the same routes with the caller's own credentials.

## What was run locally

The Rust image was built with `docker build -f Dockerfile.rust .`, and its `api` and `extraction`
binaries were run with no nginx against a throwaway PostgreSQL and MinIO, both with TLS on, using
the platform's variable names and its database URL form (`sslmode=verify-full&sslrootcert=/app/certs/rds-global-bundle.pem`,
with a test CA in that file's place), roles created the way the platform's provisioning task creates
them, and `HOSTED_DEMO=true`. With `curl` and a cookie jar: load `/`, sign in with the supplied
`DEMO_PASSWORD` (the README's password is refused), create an application, upload
`sample-docs/cedar-ridge-financial-statement.pdf`, wait for the worker's figures, download the file
and compare it byte for byte. Also: each refusal listed under "Settings".
