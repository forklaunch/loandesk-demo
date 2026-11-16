# LoanDesk

Internal web app for a commercial lending team: relationship managers create
loan applications and upload supporting documents, a background worker pulls
key figures out of the PDFs, and credit officers approve or decline.

The backend is a [ForkLaunch](https://forklaunch.com) application: a Rust API
service and a Rust worker (`.forklaunch/manifest.toml`).

## Run it

```sh
docker compose up --build
```

Then open **http://127.0.0.1:41080**. The first build compiles the Rust
services and takes several minutes.

- Only the web UI is published, on `127.0.0.1:41080`. The API, PostgreSQL and
  the object store are reachable only inside the Compose network.
- To use another port: `WEB_PORT=41090 docker compose up --build`.
- To stop: `docker compose down`. To also wipe the database, documents and
  generated secrets (and get fresh demo data next time): `docker compose down -v`.
- No `.env` file is needed and the repository contains no secrets: database
  passwords, the field-encryption key and storage keys are generated on first
  start (`scripts/demo-secrets.sh`).
- Optional Grafana/Loki/Tempo/Mimir stack: see the `observability` profile at
  the bottom of `docker-compose.yaml`.

## Demo logins

Created on first start, when the database is empty. Password for all of them:
`loandesk-demo-2026` (the Compose default for `DEMO_PASSWORD`; local demo only,
seeding is refused when `APP_ENV=production`).

| Email                        | Name         | Role                 |
| ---------------------------- | ------------ | -------------------- |
| `maya.chen@loandesk.demo`    | Maya Chen    | Relationship manager |
| `daniel.ortiz@loandesk.demo` | Daniel Ortiz | Relationship manager |
| `priya.raman@loandesk.demo`  | Priya Raman  | Credit officer       |
| `tom.becker@loandesk.demo`   | Tom Becker   | Credit officer       |
| `admin@loandesk.demo`        | Alex Morgan  | Admin                |

All seeded borrowers, owners, SSNs and EINs are fictional (SSNs are in the
900–999 range, which is never issued; EIN prefixes are ones the IRS does not use).

### A five-minute walkthrough

1. Sign in as **Maya**. She sees her four applications; try the search box
   with `dental`, `whitfield` (an owner) or `07-4410382` (an EIN).
2. Open **Cedar Ridge Veterinary Clinic**. It has no documents, so there is no
   risk flag yet. Upload `sample-docs/cedar-ridge-financial-statement.pdf`.
   Within a couple of seconds the worker fills in revenue, net income and
   total debt, and the risk summary appears.
3. Note that Maya has no approve/decline buttons.
4. Sign in as **Priya**. She sees every application, and can approve or decline
   Cedar Ridge with a note.
5. Sign in as **admin** to add a user, change someone's role, disable an account, or go through
   the access review at the bottom of **Users** and record it.
6. Under **Security**, anyone can turn on two-factor sign-in with an authenticator app, or
   change their password (every other session of theirs is signed out).

## Who can do what

| | Relationship manager | Credit officer | Admin |
| --- | --- | --- | --- |
| See applications | Only their own | All | None |
| Create applications, upload documents | Yes (own applications) | No | No |
| Download documents, view full SSN | Own applications | All | No |
| Approve / decline | No | Yes, except applications they created | No |
| Add users, set roles, disable and re-enable accounts | No | No | Yes (not their own role or account) |
| Access review, audit trail (`GET /api/audit-log`) | No | No | Yes |
| Change own password, two-factor sign-in | Yes | Yes | Yes |

Roles are declared on each route's contract (`src/api/api/controllers/`) and
enforced by one policy (`src/api/api/policy.rs`). "Cannot approve their own
applications" is enforced three ways: only credit officers can decide; nobody
can decide an application they created (this matters when a relationship
manager is later made a credit officer); and the database has a constraint for
the same rule.

## Test it

```sh
cargo test --workspace                 # unit, contract, classification and repository checks (no services needed)
cd frontend && npm ci && npm run build # type-check and build the front end

docker compose up --build -d --wait    # then, against the running stack:
python3 scripts/e2e.py                 # end-to-end checks: flows and who is refused
sh scripts/verify-storage.sh           # ciphertext at rest, append-only audit trail, row-level security
sh scripts/verify-operations.sh        # after e2e.py: nothing personal in any log, readiness, graceful shutdown
python3 scripts/load-test.py           # main read paths hold the latency objective at 32 requests/s
python3 scripts/verify-lifecycle.py    # retention purge, access review, password change, rate limits, key rotation
```

`verify-operations.sh` stops and restarts the object store, the API and the worker while it checks
them. `verify-lifecycle.py` runs last: it ages records in the database, runs the worker's scheduled
job by hand, and rotates the stack's encryption key from start to finish (the stack is on a new key
afterwards).

`docs/controls.md` lists every control a reviewer asks about, the code that implements it and the
test or check above that proves it. `docs/needs-a-person.md` lists what only a person can decide or do.

## How it fits together

`docs/architecture.md` has the diagram, the request path and a table of where
each control is enforced. In short:

- `src/api` – ForkLaunch Rust service: typed route contracts, cookie sessions,
  role policy, field encryption, audit trail.
- `src/extraction` – ForkLaunch worker on the leased PostgreSQL queue: reads a
  PDF from object storage, finds the figures, saves them. Failures that can
  pass are retried up to 3 times, then kept as dead letters; unreadable PDFs
  are marked "could not be read". It also runs the hourly housekeeping job
  (`lifecycle.rs`): the retention purge, stale accounts, and the access-review
  and key-rotation reminders.
- `src/shared` – risk formulas, the figure parser, and what an error may say in a log.
- `frontend` – React + TypeScript (Vite), served by nginx in the Compose stack. Its build is also
  committed as `src/api/web-dist` and compiled into the API, which serves it wherever there is no
  web server in front (`docs/hosting.md`); after changing the front end run
  `npm run build:embed` in `frontend/` and commit the result. Errors caught in the
  browser are reported to `POST /api/client-errors` (class and stack frames
  only, never the message) and logged as `browser_error`.
- `src/core`, `src/monitoring`, `src/client-sdk` – generated ForkLaunch
  TypeScript workspace libraries. The app does not import them: nothing in
  `src/api`, `src/extraction` or `frontend` depends on them, and they are not
  in any image. `src/monitoring` also holds the dashboards, collector config
  and alert rules, which are used.

More: `docs/controls.md`, `docs/needs-a-person.md`, `docs/deployment.md`, `docs/hosting.md`,
`docs/migration.md`, `docs/data-classification.md` (classification and
retention), `docs/data-sharing.md`, `docs/slo.md`, `docs/runbooks/`,
`docs/adr/`, `CHANGELOG.md`.

## Things you should know before relying on it

**The risk summary is a placeholder, not credit policy.** With only three
figures available, it is computed as (see `src/shared/src/risk.rs`):

- Total debt = existing debt from documents + requested amount
- Annual debt service = level payment on total debt at 8% over 10 years
- DSCR = net income / annual debt service
- Leverage = total debt / annual revenue
- High if DSCR < 1.10 or leverage > 1.00; low if DSCR ≥ 1.50 and leverage ≤ 0.50;
  otherwise medium. No flag is shown until all three figures exist.

**Figure extraction is simple label matching** (`src/shared/src/figures.rs`).
It works on PDFs that contain text; it does not OCR scans, and it will miss
documents that word things differently. When several documents are uploaded, a
figure found in a later one replaces the earlier value.

**Sensitive data.** Owner names, SSNs, dates of birth, the purpose text,
decision notes and file names are encrypted field by field. Opening an
application, downloading a document and every change are written to the
append-only `audit_log` table. Admins read it through `GET /api/audit-log`
(ids and action names only; there is no screen for it yet).

**Records are deleted when their retention period ends.** The worker's hourly
job deletes applications past their period together with their owners,
documents and stored PDFs, and writes an audit entry for each. The periods in
`docker-compose.yaml` (7 years after a decision, 3 years for applications never
decided) are **demo defaults, not a policy**: see `docs/data-classification.md`.
Accounts unused for 90 days (also a demo default) are disabled the same way.

**Full SSNs: one audited way out.** Lists, the application view and the export
carry only an SSN's last four digits. A full SSN leaves the server through
exactly one route, `POST /api/applications/{id}/owners/{owner_id}/reveal-ssn`,
which is what the **Show** link next to an owner calls. It is limited to the
people who may see that application (the relationship manager who created it,
and credit officers; never admins), returns one owner's number per call, and
commits an `ssn_revealed` audit entry before the number is sent: if the entry
cannot be written, nothing is returned. The number is shown on screen until
**Hide** is clicked or the page is left; it is not stored in the browser.
Nothing else returns a full SSN, and three checks hold that line:
`a_full_ssn_leaves_the_server_only_through_the_audited_reveal_endpoint`
(`src/api/__test__/http.rs`, over every route's response schema),
`no_stored_model_keeps_a_readable_ssn` (`src/api/__test__/persistence.rs`), and
`scripts/e2e.py` (the creator succeeds, another manager gets 404, an admin 403).

**Logs carry ids, not data.** The services log ids, fixed words, counts and
booleans. Errors from the database driver are logged as an SQLSTATE code and
constraint name only, because the driver's own text can quote a row. A test
reads every log statement in the source (`src/api/__test__/logging.rs`), and
`scripts/verify-operations.sh` searches the logs of the running stack for the
demo owners' names, SSNs, dates of birth, free text, file names and passwords.
The web server's access log records paths without query strings. Passwords are Argon2id hashes. Sessions are
HttpOnly, SameSite=Strict cookies that last 8 hours; signing out, changing the
password, a role change and disabling an account end them at once. Five failed
sign-ins lock an account for 15 minutes; sign-ins are also limited per client
address, and every user to 600 requests and 60 expensive operations a minute
(HTTP 429 beyond that). Requests with fields the API does not declare are
refused. Uploads must be PDFs of at most 20 MB.
Because names and purpose are encrypted, search matches them by whole word or
by the first three or more letters of a word, not by any fragment.

**The Compose stack is for one machine.** For the public demo copy on ForkLaunch
(`HOSTED_DEMO=true`: demo logins with a password of its own, HTTPS and TLS required, files up to
4 MB sent through the API, a banner on every screen) see `docs/hosting.md`. For a real deployment see
`docs/deployment.md`: with `APP_ENV=production` the API refuses to start with
generated demo secrets, demo seeding, insecure cookies, a plaintext database
connection or a non-HTTPS origin. With seeding off there are no users, so the
first admin has to be inserted into the `users` table by hand; there is no
bootstrap command for that yet.

**Not built:** editing applications, deleting one application on request,
password reset (a user who forgets their password needs a new account), single
sign-on, an audit-log screen, field encryption of staff email and name, backups
(a platform setting). `docs/controls.md` ends with the full list and the reasons.

## Sample documents

`sample-docs/` holds three small PDFs for the Cedar Ridge application.
Regenerate them with:

```sh
docker compose run --rm --no-deps --entrypoint /app/bin/api -u "$(id -u)" -v "$PWD/sample-docs:/out" api gen-samples /out
```
