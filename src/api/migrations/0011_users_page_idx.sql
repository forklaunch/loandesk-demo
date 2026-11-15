-- no-transaction
-- The user list: ordered by (full_name, id) with a keyset cursor.
--
-- Built CONCURRENTLY: writes to `users` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY users_page_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_page_idx
    ON users (tenant_id, full_name, id);
