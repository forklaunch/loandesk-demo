# Needs a person

What the repository cannot do for itself. Each item is a decision, a setting outside the
repository, or an action only a named person can take. Nothing below has been done or assumed done.

## Decisions

| # | Decide | Who | What is in place meanwhile |
| --- | --- | --- | --- |
| 1 | **Retention periods.** How long decided applications, never-decided applications and their documents are kept (demo defaults: 7 years after the decision, 3 years after creation, 24 hours for unfinished uploads). Set `RETENTION_DECIDED_APPLICATION_DAYS`, `RETENTION_UNDECIDED_APPLICATION_DAYS`, `RETENTION_UNFINISHED_UPLOAD_HOURS` | Lending team's records owner, with counsel | The purge job runs with the demo defaults. Set a period to `0` to keep that record type until it is decided |
| 2 | **Audit trail retention.** How long audit entries are kept, and who may remove older ones | Records owner, security | Entries are never deleted; the table refuses deletes |
| 3 | **Backup retention**, and whether a purged record may remain in backups until they expire | Records owner, platform owner | Not configured: no deployment exists |
| 4 | **Deletion on request.** Whether one application may be deleted before its period ends when a borrower asks | Counsel | Not built |
| 5 | **Access review interval and stale-account period** (demo defaults: every 90 days; disabled after 90 unused days). Set `ACCESS_REVIEW_INTERVAL_DAYS`, `ACCESS_STALE_DAYS`, `ACCESS_STALE_ACTION` | Access-control owner | The demo defaults are enforced |
| 6 | **Key rotation period** (demo default: 365 days). Set `ENCRYPTION_KEY_MAX_AGE_DAYS` | Security owner | The worker warns after 365 days |
| 7 | **Service level objectives** in `docs/slo.md` | The team that runs LoanDesk | Proposed values; the alerts use them |
| 8 | **The open advisory** RUSTSEC-2026-0459 (`libsqlite3-sys`, in the lockfile but in no binary; `docs/controls.md`): accept it as a documented exception in `cargo audit` until `sqlx` ships a fix, or keep the dependency audit failing | Security owner | `cargo audit` reports it and fails; a CI step proves the crate is not built |
| 9 | **Rate limits** (60 sign-ins a minute per address, 600 requests and 60 expensive operations a minute per user) against real usage | Engineering, after a week of production metrics | The defaults are enforced |

## Actions outside the repository

| # | Do | Who | Why it cannot be done here |
| --- | --- | --- | --- |
| 10 | **Replace the placeholder teams in `.github/CODEOWNERS`** (`@loandesk/engineering`, `backend`, `security`, `frontend`, `platform`, `credit-policy`) with real GitHub teams | Repository admin | The teams are an organization's, not the repository's |
| 11 | **Apply the branch protection**: `sh scripts/apply-branch-protection.sh OWNER/REPO` (after 10) | Repository admin | Needs admin rights on GitHub; the ruleset is declared in `.github/rulesets/main.json` |
| 12 | **Do the access reviews**: open Users as an admin, go through the accounts, press "Record that I reviewed access" (`docs/runbooks/access-review.md`) | An admin, every interval | A review is a person looking; the system records that they did |
| 13 | **Rotate the key when due**, following `docs/runbooks/key-rotation.md` | Platform owner, security | Needs the secret manager |
| 14 | **Name the alert destinations**: create the on-call service key and the team webhook, mount them as the two files `src/monitoring/alerts/alertmanager.yaml` reads, and load that file and `loandesk.rules.yaml` into the platform's Alertmanager, Mimir and Loki rulers | The team that runs LoanDesk | Who is on call is an organizational fact |
| 15 | **Connect an error-tracking backend** to the OTLP stream if one is wanted beyond logs and alerts | Engineering | A vendor choice and an account |
| 16 | **Create the environments** (staging, production) on the platform: at least two API instances with autoscaling, managed PostgreSQL with encrypted automated backups, a private bucket with default encryption and versioning, secrets in the secret manager, HTTPS origin | Platform owner | No deployment has been made (`docs/deployment.md`) |
| 17 | **Require an approval for production deploys** (a protected environment or the platform's approval step), and deploy staging first | Platform owner | No pipeline or environment exists |
| 18 | **Create the first admin** in a fresh production database: there is no bootstrap command (README) | Platform owner | A deliberate manual step until a bootstrap flow exists |
| 19 | **Rehearse a restore and a rollback** on staging (`docs/runbooks/backup-restore.md`, `docs/runbooks/rollback.md`) | Platform owner | Needs an environment |
| 20 | **Agreements**: the contract with the hosting platform as the only sub-processor (`docs/data-sharing.md`), and any notice or consent the lender owes borrowers for processing their application | Counsel | Organizational |

## Policies

No policy document, sign-off, training record, risk assessment or vendor review exists in this
repository, and none is implied by anything in it. Those belong in the organization's compliance
system.
