-- no-transaction
-- Removing expired sessions at sign-in.
--
-- Built CONCURRENTLY: writes to `sessions` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY sessions_expires_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS sessions_expires_idx
    ON sessions (expires_at);
