-- no-transaction
-- The retention job removes upload grants that were never completed.
--
-- Built CONCURRENTLY: writes to `documents` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY documents_uploading_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS documents_uploading_idx
    ON documents (tenant_id, uploaded_at)
    WHERE status = 'uploading';
