# Runbook: deploy

Follow "Order of a release" in `docs/deployment.md`. Checks after each step:

- Migration job exits 0 and logs `event=migration_complete` with `legacy_rows_after=0`.
- API logs `event=startup` with the expected `version`; `GET /health` returns 200 with
  `database` and `object_store` both `ok`.
- Worker logs `event=startup`; `GET /ready` returns 200.
- `python3 scripts/e2e.py` passes against staging (it creates data: never run it against production).

Local: `docker compose up --build -d --wait && python3 scripts/e2e.py && sh scripts/verify-storage.sh`.
