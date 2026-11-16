# Where personal data goes

The single list of every system outside the LoanDesk processes that receives data from it.
LoanDesk calls no third-party API: no email or SMS provider, no payment processor, no analytics,
no credit bureau, no AI service. `this_product_has_no_payments_webhooks_realtime_connections_or_caches`
(`src/api/__test__/controls.rs`) fails if a dependency for one is added without this page changing.

| Recipient | What it receives | How | Configured by |
| --- | --- | --- | --- |
| PostgreSQL (the platform's managed database) | Every record. Personal fields as ciphertext; business details and staff directory in clear, under tenant row-level security | TLS with certificate verification (`sslmode=verify-full`); plaintext refused in production | `DATABASE_URL`, `MIGRATION_DATABASE_URL` |
| Object storage (S3 on the platform; MinIO locally) | Uploaded PDFs, under the tenant's prefix, encrypted at rest | HTTPS (`S3_URL` must be `https://` in production; AWS S3 endpoints otherwise). Browsers use short-lived signed grants | `S3_BUCKET`, `S3_REGION`, `S3_URL` |
| OTLP collector (the platform's telemetry) | Logs, metrics and traces: ids, fixed event names, counts, durations. No names, SSNs, dates of birth, free text, file names or credentials (`src/api/__test__/logging.rs`, `scripts/verify-operations.sh`) | OTLP over HTTP(S) | `OTEL_EXPORTER_OTLP_ENDPOINT` |
| The user's browser | What the signed-in user may see | HTTPS, same origin | `PUBLIC_ORIGIN` |

Outgoing webhooks, exports to other systems and data sales: none. The only export is
`GET /api/applications/{id}/export`, a JSON document returned to a signed-in user who may already
see that application; it is audited (`application_exported`) and excludes full SSNs.

The hosting platform (ForkLaunch on AWS) is therefore the only sub-processor. NEEDS A PERSON: the
agreement with it, and where its telemetry is stored, are organizational facts this repository
cannot state (`docs/needs-a-person.md`).
