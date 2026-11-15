# LoanDesk

Internal web app for a commercial lending team: relationship managers create
loan applications and upload supporting documents, a background worker pulls
key figures out of the PDFs, and credit officers approve or decline.

## Run it

```sh
docker compose up --build
```

Then open **http://localhost:41080**. The first build compiles the Rust
services and takes a few minutes.

- Only the web UI is published, on `127.0.0.1:41080`. The API and Postgres are
  reachable only inside the Compose network.
- To use another port: `WEB_PORT=41090 docker compose up --build`.
- To stop: `docker compose down`. To also wipe the database and uploaded
  documents (and get fresh demo data next time): `docker compose down -v`.

## Demo logins

Created on first start, when the database is empty. Password for all of them:
`loandesk-demo-2026`

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
   total debt, and the risk summary shows **Medium**.
3. Note that Maya has no approve/decline buttons.
4. Sign in as **Priya**. She sees every application, and can approve or decline
   Cedar Ridge with a note.
5. Sign in as **admin** to add a user or change someone's role.

## Who can do what

| | Relationship manager | Credit officer | Admin |
| --- | --- | --- | --- |
| See applications | Only their own | All | None |
| Create applications, upload documents | Yes (own applications) | No | No |
| Download documents, view full SSN | Own applications | All | No |
| Approve / decline | No | Yes, except applications they created | No |
| Add users, set roles | No | No | Yes (not their own role) |

Two of these are my reading of the brief rather than something it spelled
out, and are easy to change in `backend/api/src/applications.rs`
(`visibility_filter`):

- Relationship managers see only their own applications, not the whole team's.
- Admins manage users but cannot see borrower data.

"Cannot approve their own applications" is enforced three ways: only credit
officers can decide; nobody can decide an application they created (this
matters when an RM is later made a credit officer); and the database has a
constraint for the same rule.

## How it fits together

```
browser ──> web (nginx: React app, proxies /api) ──> api (Axum) ──> Postgres
                                                       │               ▲
                                                       └─ docs volume ─┤
                                                          worker ──────┘
```

- `backend/api` – REST API. Runs database migrations and demo seeding on start.
- `backend/worker` – polls the `documents` table for pending uploads, extracts
  PDF text, finds the figures, and saves them on the document and application.
  Failed documents are retried up to 3 times, then marked "could not be read".
- `backend/shared` – risk formulas and the figure parser (with unit tests:
  `cd backend && cargo test -p shared`).
- `frontend` – React + TypeScript (Vite), served by nginx.
- Uploaded PDFs live on a Docker volume shared by the API and worker.

## Things you should know before relying on it

**The risk summary is a placeholder, not credit policy.** With only three
figures available, it is computed as (see `backend/shared/src/risk.rs`):

- Total debt = existing debt from documents + requested amount
- Annual debt service = level payment on total debt at 8% over 10 years
- DSCR = net income / annual debt service
- Leverage = total debt / annual revenue
- High if DSCR < 1.10 or leverage > 1.00; low if DSCR ≥ 1.50 and leverage ≤ 0.50;
  otherwise medium. No flag is shown until all three figures exist.

**Figure extraction is simple label matching.** The worker looks for lines
that start with labels such as "Total revenue", "Gross receipts or sales",
"Net income", "Total debt" followed by an amount
(`backend/shared/src/figures.rs`). It works on PDFs that contain text; it does
not OCR scans, and it will miss documents that word things differently. When
several documents are uploaded, a figure found in a later one replaces the
earlier value. The UI shows what was read from each document so it can be
checked against the source; there is no screen to correct a figure by hand.

**Handling of sensitive data**

- Owner SSNs are encrypted in the database (AES-256-GCM); screens show the
  last four digits, and showing a full SSN is written to the `audit_log` table.
  Logins, uploads, downloads, decisions and user changes are logged there too
  (there is no screen for the audit log yet; query the table).
- Passwords are hashed with Argon2id. Sessions are HttpOnly, SameSite=Strict
  cookies that last 8 hours. Five failed sign-ins lock an account for 15 minutes.
- Uploads must be PDFs of at most 20 MB.
- Dates of birth and the uploaded PDFs themselves are **not** encrypted by the
  app; they rely on whatever disk/database encryption the host provides.

**The defaults are for local demos.** `docker-compose.yml` ships a known
database password, a known SSN encryption key and the demo logins above. Before
putting real borrower data in it, copy `.env.example` to `.env`, set
`POSTGRES_PASSWORD` and `DATA_ENCRYPTION_KEY` (`openssl rand -base64 32`), set
`SEED_DEMO_DATA=false`, put it behind HTTPS and set `APP_ENV=production` and
`COOKIE_SECURE=true`. With `APP_ENV=production` the API refuses to start with
the demo key, with demo seeding on, or without secure cookies. With seeding
off there are no users, so the first admin has to be inserted into the `users`
table by hand; there is no bootstrap command for that yet.

**Not built** (not asked for, but you will likely want them): editing or
deleting applications, password change/reset, deactivating users, single
sign-on, an audit-log screen, backups.

## Sample documents

`sample-docs/` holds three small PDFs for the Cedar Ridge application (a
financial statement, a tax return and a bank statement, which has no key
figures in it). Regenerate them with:

```sh
docker compose run --rm --no-deps -u "$(id -u)" -v "$PWD/sample-docs:/out" api loandesk-api gen-samples /out
```
