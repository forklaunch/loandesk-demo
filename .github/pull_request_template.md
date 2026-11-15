## What changed

<!-- One or two sentences. Link the issue or report-card finding. -->

## How it was tested

- [ ] `cargo test --workspace`
- [ ] `python3 scripts/e2e.py` against `docker compose up --build`, then `sh scripts/verify-storage.sh` and `sh scripts/verify-operations.sh`
- [ ] Front end: `npm run build` in `frontend/`
- Other checks (describe):

## Risk

- Does it touch authorization, sessions, encryption or the audit trail?
- Does it add a table or column? (Every field needs a classification in `src/api/persistence/entities/`.)
- Does it add a migration? (Never edit an applied migration; add a new one.)

## Rollback

<!-- How to undo this if it goes wrong: revert only, or a data step too? -->

## Changelog

- [ ] `CHANGELOG.md` updated under "Unreleased"
