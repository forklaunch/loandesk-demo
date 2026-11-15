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
5. Sign in as **admin** to add a user or change someone's role.
6. Under **Security**, anyone can turn on two-factor sign-in with an authenticator app.

## Who can do what

| | Relationship manager | Credit officer | Admin |
| --- | --- | --- | --- |
| See applications | Only their own | All | None |
| Create applications, upload documents | Yes (own applications) | No | No |
| Download documents, view full SSN | Own applications | All | No |
| Approve / decline | No | Yes, except applications they created | No |
| Add users, set roles | No | No | Yes (not their own role) |

Roles are declared on each route's contract (`src/api/api/controllers/`) and
enforced by one policy (`src/api/api/policy.rs`). "Cannot approve their own
applications" is enforced three ways: only credit officers can decide; nobody
can decide an application they created (this matters when a relationship
manager is later made a credit officer); and the database has a constraint for
the same rule.

## Test it

```sh
cargo test --workspace                 # unit, contract and classification tests (no services needed)
cd frontend && npm ci && npm run build # type-check and build the front end

docker compose up --build -d --wait    # then, against the running stack:
python3 scripts/e2e.py                 # end-to-end checks: flows and who is refused
sh scripts/verify-storage.sh           # ciphertext at rest, append-only audit trail, row-level security
sh scripts/verify-operations.sh        # after e2e.py: nothing personal in any log, readiness, graceful shutdown
```

`verify-operations.sh` stops and restarts the object store, the API and the worker while it checks them.

## How it fits together

`docs/architecture.md` has the diagram, the request path and a table of where
each control is enforced. In short:

- `src/api` – ForkLaunch Rust service: typed route contracts, cookie sessions,
  role policy, field encryption, audit trail.
- `src/extraction` – ForkLaunch worker on the leased PostgreSQL queue: reads a
  PDF from object storage, finds the figures, saves them. Failures that can
  pass are retried up to 3 times, then kept as dead letters; unreadable PDFs
  are marked "could not be read".
- `src/shared` – risk formulas, the figure parser, and what an error may say in a log.
- `frontend` – React + TypeScript (Vite), served by nginx. Errors caught in the
  browser are reported to `POST /api/client-errors` (class and stack frames
  only, never the message) and logged as `browser_error`.
- `src/core`, `src/monitoring`, `src/client-sdk` – generated ForkLaunch
  TypeScript workspace libraries. The app does not import them: nothing in
  `src/api`, `src/extraction` or `frontend` depends on them, and they are not
  in any image. `src/monitoring` also holds the dashboards, collector config
  and alert rules, which are used.

More: `docs/deployment.md`, `docs/migration.md`, `docs/data-classification.md`,
`docs/runbooks/`, `docs/adr/`, `CHANGELOG.md`.

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
append-only `audit_log` table (there is no screen for it yet).

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
HttpOnly, SameSite=Strict cookies that last 8 hours. Five failed sign-ins lock
an account for 15 minutes. Uploads must be PDFs of at most 20 MB.
Because names and purpose are encrypted, search matches them by whole word or
by the first three or more letters of a word, not by any fragment.

**The Compose stack is for one machine.** For a real deployment see
`docs/deployment.md`: with `APP_ENV=production` the API refuses to start with
generated demo secrets, demo seeding, insecure cookies, a plaintext database
connection or a non-HTTPS origin. With seeding off there are no users, so the
first admin has to be inserted into the `users` table by hand; there is no
bootstrap command for that yet.

**Not built:** editing or deleting applications, password change/reset,
deactivating users, single sign-on, an audit-log screen, a retention purge for
applications, backups (a platform setting).

## Sample documents

`sample-docs/` holds three small PDFs for the Cedar Ridge application.
Regenerate them with:

```sh
docker compose run --rm --no-deps --entrypoint /app/bin/api -u "$(id -u)" -v "$PWD/sample-docs:/out" api gen-samples /out
```
