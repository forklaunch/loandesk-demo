-- no-transaction
-- Audit trail by time.
--
-- Built CONCURRENTLY: writes to `audit_log` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY audit_log_at_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS audit_log_at_idx
    ON audit_log (tenant_id, at);
