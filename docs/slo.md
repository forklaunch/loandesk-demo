# Service level objectives

Proposed targets; the team that runs LoanDesk should confirm them.

| Journey | Objective | Measured by |
| --- | --- | --- |
| API availability | 99.5% of requests are not 5xx, per 30 days | `http.server.request.count` by status |
| API latency | 95% of requests under 1 s | `http.server.request.duration` |
| Document reading | 95% of uploaded documents processed within 2 minutes | `loandesk.extraction.job.duration_ms`, queue depth |
| Dead letters | 0 unresolved for more than one working day | `loandesk.extraction.queue.dead_letters` |

Alert rules for these are in `src/monitoring/alerts/loandesk.rules.yaml`.
