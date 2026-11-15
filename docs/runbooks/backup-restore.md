# Runbook: back up and restore

**Database.** Managed PostgreSQL: enable automated encrypted snapshots with a bounded retention
(for example 35 days) and take a manual snapshot before each release. By hand:

```sh
pg_dump --format=custom --no-owner "$MIGRATION_DATABASE_URL" | age -r <recipient> > loandesk-$(date +%F).dump.age
age -d -i <identity> loandesk-YYYY-MM-DD.dump.age | pg_restore --no-owner --role=loandesk_owner -d "$MIGRATION_DATABASE_URL"
```

Dumps contain ciphertext for personal fields but business data in clear: always encrypt them.

**Documents.** Enable bucket versioning; restore by copying the previous version.

**Keys.** A database backup is unreadable without the encryption keys that were active when it was
taken. Keep every retired key in the secret manager (as `ENCRYPTION_PREVIOUS_KEYS_JSON`) for as
long as any backup made under it is kept.

After a restore: `sh scripts/verify-storage.sh`, sign in, open an application, reveal one SSN.

Backup schedules and retention are platform settings: not configured by this repository.
