# Security events

The services write a named `event` for everything below (JSON logs, shipped over OTLP). The alert
rules in `src/monitoring/alerts/loandesk.rules.yaml` count them; `alertmanager.yaml` beside it routes
`page` alerts to the on-call receiver and `ticket` alerts to the team receiver. Each log line has a
`request_id`; the same id is on the audit entries of that request (`GET /api/audit-log`, admins).

## Failed sign-ins

`login_failed` (wrong password, unknown email or wrong second factor; the user id when the email
exists) and `login_rate_limited` (the account is locked for 15 minutes after five failures).
A burst against many accounts is password spraying: find the client address in the web server's
access log by time, block it at the load balancer, and check for a `login` event from the same
address afterwards. A burst against one account: tell the user, and have them change their password.

## Rate limits

`rate_limited`, with `limit` = `login` (per client address), `requests` or `expensive` (per user).
One user over the limit is usually a script or a stuck browser tab: the `user_id` is on the line.
Many users at once means the limits are too low for real use: raise `RATE_LIMIT_*` (`.env.example`).

## Invalid sessions

`invalid_session`: a request carried a session cookie that is unknown, expired or revoked. A few
after a deploy, a role change or a password change are normal (those end sessions). A steady stream
is someone replaying or guessing tokens: tokens are 256 random bits and only their hash is stored,
so guessing cannot succeed, but block the source.

## Permission denials

`permission_denied`, with the user, their role, the action and the record. An application id that
"does not exist" for the caller is logged here too. One user probing many ids is the pattern to
look for: disable the account (`POST /api/users/{id}/disable`) while you ask.

## Disabled account

`login_refused_disabled`: the correct password was entered for a disabled account. Either a
colleague who should be re-enabled (`POST /api/users/{id}/enable`), or a credential that is still
in someone's hands after the account was closed.

## Password change

`password_change_refused`: a signed-in session tried to change the password with a wrong current
password. Repeated, it suggests the session is not in its owner's hands: disable the account, which
ends its sessions, and contact the user.

## Bulk access

`ssn_revealed`, `application_exported`, `document_downloaded`. Each is one audited action by one
user on one record. Volume far above normal work is the signal. The audit trail lists exactly
which records: filter `GET /api/audit-log` by the user id.

## Account changes

`user_created`, `user_role_changed`, `user_disabled`, `user_enabled`. Every one raises a ticket so
that a second person sees each change to who has access. Confirm with the admin named on the line.

## Cross-tenant storage key

`cross_tenant_storage_key`: a document row pointed outside the tenant's prefix in object storage.
The request was refused. This cannot happen through the application; treat it as tampering with
the database and start the incident process.
