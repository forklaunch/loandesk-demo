# Service level objectives

Proposed targets; the team that runs LoanDesk should confirm them.

| Journey | Objective | Error budget (30 days) | Measured by |
| --- | --- | --- | --- |
| API availability (sign in, list, open, create, decide) | 99.5% of requests are not 5xx | 0.5% of requests | `http.server.request.count` by status |
| API latency | 95% of requests under 1 s | 5% of requests | `http.server.request.duration` |
| Document reading | 95% of uploaded documents processed within 2 minutes | 5% of documents | `loandesk.extraction.job.duration_ms`, queue depth |
| Dead letters | 0 unresolved for more than one working day | none | `loandesk.extraction.queue.dead_letters` |
| Screens work | No sustained stream of browser errors | 10 reports in 10 minutes | `browser_error` log events |

## Alerts

`src/monitoring/alerts/loandesk.rules.yaml`. The availability objective is alerted on by how fast
its error budget is being spent (group `loandesk-slo-burn`):

| Alert | Fires when | Meaning | Action |
| --- | --- | --- | --- |
| `LoanDeskErrorBudgetFastBurn` | error ratio above 14.4 x 0.5% over both 1 hour and 5 minutes | 2% of the month's budget gone in an hour | page |
| `LoanDeskErrorBudgetSlowBurn` | error ratio above 6 x 0.5% over both 6 hours and 30 minutes | 5% of the month's budget gone in six hours | page |
| `LoanDeskErrorBudgetTrend` | error ratio above 1 x 0.5% over both 3 days and 6 hours | on course to miss the objective | ticket |

The other groups hold threshold alerts for latency, the queue, dead letters, dependency calls,
security events (`docs/runbooks/security-events.md`), the scheduled job, the access review and key
rotation reminders, panics and browser errors. Every alert is `severity: page` or `severity: ticket`;
`src/monitoring/alerts/alertmanager.yaml` routes the first to the on-call receiver and the second
to the team receiver. Which rotation and which channel those are is for the operating team to fill
in (`docs/needs-a-person.md`).

## Expected capacity

`scripts/load-test.py` drives the main read paths (application list, one application with its
personal fields decrypted, search over encrypted text) with four signed-in users at 8 requests a
second each, and fails if any response is a server error, if the 95th percentile is over the
1-second objective, or if the stack does not keep up. It runs in CI after the end-to-end checks.

Measured on the local Compose stack (one API container limited to one CPU and 512 MB, PostgreSQL
and MinIO beside it on a laptop), 2026-11-15:

| | Requests | p50 | p95 |
| --- | --- | --- | --- |
| List | 469 | 5.5 ms | 15.4 ms |
| One application | 150 | 11.5 ms | 36.5 ms |
| Search | 21 | 6.5 ms | 39.9 ms |
| All, 32 requests/s for 20 s | 640 | 6.6 ms | 24.7 ms |

This is a floor the API is expected to hold, not its limit: the load stays inside the per-user
rate limit (600 requests a minute), which caps what one user can ask for before hardware does.
A lending team of some dozens of people is far below it. Capacity beyond this has not been
measured.
