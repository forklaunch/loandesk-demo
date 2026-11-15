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

The other groups hold threshold alerts for latency, the queue, dead letters, security events and
browser errors. Who is paged (Alertmanager receiver, on-call rotation) is configured where the rules
are loaded and is not decided in this repository.
