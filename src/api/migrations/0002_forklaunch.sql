-- Moves the LoanDesk schema onto the ForkLaunch storage model.
--
-- Run by `api --migrate` with the migration (schema owner) role. That command
-- sets `forklaunch.tenant_id` on its connection first, so existing rows are
-- adopted into the configured tenant; running this file without it fails
-- loudly rather than leaving rows without a tenant.
--
-- 0001_init.sql is unchanged. Nothing is dropped here: legacy plaintext
-- columns become nullable and are emptied by the `--migrate` re-encryption
-- step (persistence/legacy.rs), so a failed cutover can still be rolled back
-- from the same database (see docs/runbooks/rollback.md).

-- 1. Tenant scope and forced row-level security on every table -------------

ALTER TABLE users        ADD COLUMN tenant_id text;
ALTER TABLE sessions     ADD COLUMN tenant_id text;
ALTER TABLE applications ADD COLUMN tenant_id text;
ALTER TABLE owners       ADD COLUMN tenant_id text;
ALTER TABLE documents    ADD COLUMN tenant_id text;
ALTER TABLE audit_log    ADD COLUMN tenant_id text;

UPDATE users        SET tenant_id = current_setting('forklaunch.tenant_id');
UPDATE sessions     SET tenant_id = current_setting('forklaunch.tenant_id');
UPDATE applications SET tenant_id = current_setting('forklaunch.tenant_id');
UPDATE owners       SET tenant_id = current_setting('forklaunch.tenant_id');
UPDATE documents    SET tenant_id = current_setting('forklaunch.tenant_id');
UPDATE audit_log    SET tenant_id = current_setting('forklaunch.tenant_id');

DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY['users', 'sessions', 'applications', 'owners', 'documents', 'audit_log'] LOOP
    EXECUTE format('ALTER TABLE %I ALTER COLUMN tenant_id SET NOT NULL', t);
    EXECUTE format('ALTER TABLE %I ALTER COLUMN tenant_id SET DEFAULT current_setting(''forklaunch.tenant_id'')', t);
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
    EXECUTE format(
      'CREATE POLICY forklaunch_tenant_policy ON %I
         USING (tenant_id = current_setting(''forklaunch.tenant_id'', true))
         WITH CHECK (tenant_id = current_setting(''forklaunch.tenant_id'', true))', t);
  END LOOP;
END $$;

-- 2. Classified fields are stored as ciphertext ----------------------------
-- (ForkLaunch ProtectedField: AES-256-GCM under a per-tenant derived key,
--  bound to the row id and a per-field purpose, key id recorded for rotation.)

ALTER TABLE owners
    ADD COLUMN full_name_ciphertext     text,
    ADD COLUMN ssn_ciphertext           text,
    ADD COLUMN date_of_birth_ciphertext text,
    ALTER COLUMN full_name     DROP NOT NULL,
    ALTER COLUMN ssn_encrypted DROP NOT NULL,
    ALTER COLUMN ssn_last4     DROP NOT NULL,
    ALTER COLUMN date_of_birth DROP NOT NULL;

ALTER TABLE applications
    ADD COLUMN purpose_ciphertext       text,
    ADD COLUMN decision_note_ciphertext text,
    -- Keyed (HMAC) word-prefix tokens for the encrypted free text and owner
    -- names, so search still works without storing those values in clear.
    ADD COLUMN search_tokens            text[] NOT NULL DEFAULT '{}',
    -- Client-supplied Idempotency-Key; a retried create returns the first result.
    ADD COLUMN idempotency_key          text,
    ALTER COLUMN purpose DROP NOT NULL;

ALTER TABLE documents
    ADD COLUMN filename_ciphertext text,
    -- Every stored file carries a classification, like a database field.
    ADD COLUMN classification text NOT NULL DEFAULT 'pii'
        CHECK (classification IN ('none', 'pii', 'phi', 'pci')),
    -- Request that queued the extraction job, for log and trace correlation.
    ADD COLUMN request_id text,
    ALTER COLUMN filename DROP NOT NULL;

-- 'uploading': an upload grant was issued but the file is not confirmed yet.
ALTER TABLE documents DROP CONSTRAINT documents_status_check;
ALTER TABLE documents ADD CONSTRAINT documents_status_check
    CHECK (status IN ('uploading', 'pending', 'processing', 'done', 'failed'));

-- 3. Audit trail: correlation id, append-only ------------------------------

ALTER TABLE audit_log ADD COLUMN request_id text;

CREATE FUNCTION audit_log_is_append_only() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'audit_log is append-only';
END $$;

CREATE TRIGGER audit_log_no_update_or_delete
    BEFORE UPDATE OR DELETE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_log_is_append_only();
CREATE TRIGGER audit_log_no_truncate
    BEFORE TRUNCATE ON audit_log
    FOR EACH STATEMENT EXECUTE FUNCTION audit_log_is_append_only();

-- 4. Indexes ---------------------------------------------------------------
-- The indexes for foreign keys, filters and keyset pagination are built by the
-- migrations that follow (0003 to 0014), one per file, with CREATE INDEX
-- CONCURRENTLY, so that building them does not block writes to these tables.

-- 5. Two-factor sign-in ------------------------------------------------------

CREATE TABLE user_mfa (
    user_id            uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    tenant_id          text NOT NULL DEFAULT current_setting('forklaunch.tenant_id'),
    secret_ciphertext  text NOT NULL,
    confirmed_at       timestamptz
);
ALTER TABLE user_mfa ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_mfa FORCE ROW LEVEL SECURITY;
CREATE POLICY forklaunch_tenant_policy ON user_mfa
    USING (tenant_id = current_setting('forklaunch.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('forklaunch.tenant_id', true));
