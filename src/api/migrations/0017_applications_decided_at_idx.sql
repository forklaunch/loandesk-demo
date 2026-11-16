-- no-transaction
-- The retention job selects decided applications by the date they were decided.
--
-- Built CONCURRENTLY: writes to `applications` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY applications_decided_at_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS applications_decided_at_idx
    ON applications (tenant_id, decided_at)
    WHERE decided_at IS NOT NULL;
