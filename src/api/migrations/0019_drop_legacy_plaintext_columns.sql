-- Contract step of the move to field encryption (expand: 0002; backfill:
-- `api --migrate`, persistence/legacy.rs; switch: every release since 0002
-- reads and writes only the *_ciphertext columns).
--
-- Drops the columns that held owner names, SSNs (old cipher), SSN last-four,
-- dates of birth, purpose, decision notes and file names before 0002. They
-- have been empty since the backfill; nothing in the API or the worker reads
-- or writes them, so the release that is still running during a rolling deploy
-- is unaffected. `documents.attempts` and `documents.locked_at` stay: the
-- worker still writes them.
--
-- `api --migrate` applies the migrations up to 0018, runs the backfill, checks
-- that nothing readable is left, and only then applies this file. The guard
-- below is the second line of defence: run by any other tool against a
-- database that still holds plaintext, it fails instead of deleting data.
--
-- DROP COLUMN only changes the catalog (no table rewrite, no scan), and waits
-- at most five seconds for its lock instead of queueing behind long readers.
--
-- After this migration a rollback to the pre-0002 backend is by restoring the
-- backup taken before the cutover (docs/runbooks/rollback.md), not in place.
SET LOCAL lock_timeout = '5s';
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM owners WHERE full_name IS NOT NULL OR ssn_encrypted IS NOT NULL
                                     OR ssn_last4 IS NOT NULL OR date_of_birth IS NOT NULL
                                     OR ssn_ciphertext IS NULL)
     OR EXISTS (SELECT 1 FROM applications WHERE purpose IS NOT NULL OR decision_note IS NOT NULL)
     OR EXISTS (SELECT 1 FROM documents WHERE filename IS NOT NULL)
  THEN
    RAISE EXCEPTION 'legacy plaintext rows remain: run `api --migrate` (it re-encrypts them before this migration)';
  END IF;
END $$;
ALTER TABLE owners
    DROP COLUMN full_name,
    DROP COLUMN ssn_encrypted,
    DROP COLUMN ssn_last4,
    DROP COLUMN date_of_birth;
ALTER TABLE applications
    DROP COLUMN purpose,
    DROP COLUMN decision_note;
ALTER TABLE documents
    DROP COLUMN filename;
