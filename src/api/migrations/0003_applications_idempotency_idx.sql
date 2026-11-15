-- no-transaction
-- A retried create with the same Idempotency-Key finds the first result (the ON CONFLICT target).
--
-- Built CONCURRENTLY: writes to `applications` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY applications_idempotency_idx)
-- and run `api --migrate` again.
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS applications_idempotency_idx
    ON applications (tenant_id, created_by, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
