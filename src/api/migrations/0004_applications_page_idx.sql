-- no-transaction
-- Credit officers' list: every application, newest first, by keyset cursor (created_at DESC, id DESC).
--
-- Built CONCURRENTLY: writes to `applications` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY applications_page_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS applications_page_idx
    ON applications (tenant_id, created_at DESC, id DESC);
