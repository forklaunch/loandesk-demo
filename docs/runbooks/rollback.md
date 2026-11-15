# Runbook: roll back

**Code only (no migration in the release):** redeploy the previous image tag. Sessions survive.

**Release included a migration:**
1. Stop the API and worker (they drain for up to 40 s).
2. If the migration only added things (columns, indexes, tables), redeploy the previous image; it ignores them.
3. If data was transformed (`0002_forklaunch.sql` re-encrypts fields), restore the backup taken
   before the release (`backup-restore.md`) and redeploy the previous image. Work done since the
   backup is lost: say so before deciding.

Migrations are never edited after they are applied; a mistake is fixed with a new migration.
There are no automatic down-migrations.
