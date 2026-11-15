-- no-transaction
-- Search on the encrypted purpose and owner names through keyed word tokens (search_tokens @> ...).
--
-- Built CONCURRENTLY: writes to `applications` continue while the index is built.
-- CONCURRENTLY cannot run in a transaction, and a migration file is sent to
-- PostgreSQL as one query, so this file holds exactly one statement. If the
-- build is interrupted it leaves an INVALID index, and `api --migrate` refuses
-- to finish while one exists: drop it (DROP INDEX CONCURRENTLY applications_search_idx)
-- and run `api --migrate` again.
CREATE INDEX CONCURRENTLY IF NOT EXISTS applications_search_idx
    ON applications USING gin (search_tokens);
