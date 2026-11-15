# Runbook: likely failures

Logs are JSON; filter on `event` and follow one action with `request_id` (the worker logs the
`request_id` of the request that queued each job).

## API errors
`event=dependency_error` names the `operation` that failed. `GET /health` shows which dependency
is down. Database down: the API reports not-ready and is taken out of rotation; it recovers by itself.

## API down
Check the last `event=startup` / `shutdown_complete` / `panic` lines. A startup refusal in
production names the setting to fix.

## Slow requests
Statements are cancelled after `DATABASE_STATEMENT_TIMEOUT_MS`; pool waits fail after 5 s.
Look for `dependency_error` with timeouts, then database load.

## Dead letters
`event=dead_letter_jobs`. List them: `extraction --dead-letters 100`. Each `job_failed` line has the
job id (= document id), attempt and failing step. After fixing the cause: `extraction --retry <id>`.
Documents that could not be parsed are marked "Could not be read" and are not retried: the user uploads a better copy.

## Queue backlog
Worker stopped or too slow. Check `GET /ready` on the worker; add worker instances or raise `WORKER_CONCURRENCY`.

## Failed sign-ins
`event=login_failed` / `login_rate_limited` with `user_id`. Five failures lock an account for 15 minutes.
A burst across many accounts is password guessing: block the source at the edge.

## Revoke someone's access now
Change their role (admin screen): their sessions end immediately. To end every session:
`DELETE FROM sessions;` as the app role inside a tenant transaction.

## Rotate the field-encryption key
Add the new key as `ENCRYPTION_KEY_ID`/`ENCRYPTION_KEY_HEX`, move the old one into
`ENCRYPTION_PREVIOUS_KEYS_JSON`, redeploy. New writes use the new key; old rows stay readable.
Re-encrypting existing rows and rebuilding search tokens is not scripted yet.
