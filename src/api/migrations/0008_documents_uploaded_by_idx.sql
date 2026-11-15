-- no-transaction
-- Foreign key documents.uploaded_by -> users.
--
-- Built CONCURRENTLY: writes to `documents` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY documents_uploaded_by_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS documents_uploaded_by_idx
    ON documents (uploaded_by);
