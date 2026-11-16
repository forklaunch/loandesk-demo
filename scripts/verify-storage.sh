#!/bin/sh
# Checks the running Compose stack's database directly: classified columns hold
# only ciphertext, no legacy plaintext is left, the audit trail refuses edits,
# and the application role cannot bypass row-level security.
set -eu
psql() {
  docker compose exec -T postgresql sh -c \
    'PGPASSWORD="$(cat /run/demo-secrets/postgres_password)" psql -v ON_ERROR_STOP=1 -U postgres -d loandesk -At "$@"' sh "$@"
}
expect() { # label, expected, actual
  if [ "$2" = "$3" ]; then echo "ok    $1"; else echo "FAIL  $1 (expected $2, got $3)"; exit 1; fi
}
expect "every owner field is ciphertext" 0 "$(psql -c "SELECT count(*) FROM owners WHERE full_name_ciphertext !~ '^(v3|pd1)\.' OR ssn_ciphertext !~ '^(v3|pd1)\.' OR date_of_birth_ciphertext !~ '^(v3|pd1)\.'")"
expect "the pre-encryption plaintext columns no longer exist" 0 "$(psql -c "SELECT count(*) FROM information_schema.columns WHERE table_schema = 'public' AND (table_name, column_name) IN (('owners','full_name'),('owners','ssn_encrypted'),('owners','ssn_last4'),('owners','date_of_birth'),('applications','purpose'),('applications','decision_note'),('documents','filename'))")"
expect "purpose, decision notes and file names are ciphertext" 0 "$(psql -c "SELECT (SELECT count(*) FROM applications WHERE purpose_ciphertext !~ '^(v3|pd1)\.' OR decision_note_ciphertext !~ '^(v3|pd1)\.') + (SELECT count(*) FROM documents WHERE filename_ciphertext !~ '^(v3|pd1)\.')")"
expect "two-factor secrets are ciphertext" 0 "$(psql -c "SELECT count(*) FROM user_mfa WHERE secret_ciphertext !~ '^(v3|pd1)\.'")"
expect "search tokens are keyed HMAC values, not words" 0 "$(psql -c "SELECT count(*) FROM applications a WHERE EXISTS (SELECT 1 FROM unnest(a.search_tokens) t WHERE t !~ '^bi1\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]{43}$')")"
expect "every table has a NOT NULL tenant_id" 0 "$(psql -c "SELECT count(*) FROM pg_class c WHERE c.relnamespace = 'public'::regnamespace AND c.relkind = 'r' AND c.relname <> '_sqlx_migrations' AND NOT EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = c.oid AND a.attname = 'tenant_id' AND a.attnotnull)")"
expect "no index was left invalid by an interrupted build" 0 "$(psql -c "SELECT count(*) FROM pg_index i JOIN pg_class c ON c.oid = i.indexrelid WHERE c.relnamespace = 'public'::regnamespace AND NOT i.indisvalid")"
expect "queued job payloads are encrypted" 0 "$(psql -c "SELECT count(*) FROM fl_worker.jobs WHERE payload::text LIKE '%applicationId%'")"
expect "every stored file is under its tenant prefix and classified" 0 "$(psql -c "SELECT count(*) FROM documents WHERE storage_key NOT LIKE 'tenants/' || tenant_id || '/%' OR classification IS NULL")"
expect "the app role is not superuser and cannot bypass RLS" "f|f|f" "$(psql -c "SELECT rolsuper, rolbypassrls, rolcreaterole FROM pg_roles WHERE rolname = 'loandesk_app'")"
expect "row-level security is forced on every table" 0 "$(psql -c "SELECT count(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace AND relkind = 'r' AND relname <> '_sqlx_migrations' AND NOT (relrowsecurity AND relforcerowsecurity)")"
expect "another tenant sees no rows" 0 "$(psql -c "SET ROLE loandesk_app; SELECT set_config('forklaunch.tenant_id', '00000000-0000-4000-8000-000000000001', false); SELECT count(*) FROM applications" | tail -1)"
if psql -c "UPDATE audit_log SET action = 'tampered' WHERE id = (SELECT min(id) FROM audit_log)" >/dev/null 2>&1; then
  echo "FAIL  audit_log accepted an UPDATE"; exit 1
else echo "ok    audit_log refuses UPDATE"; fi
if psql -c "DELETE FROM audit_log" >/dev/null 2>&1; then
  echo "FAIL  audit_log accepted a DELETE"; exit 1
else echo "ok    audit_log refuses DELETE"; fi
expect "every audit entry has its action, time and tenant" 0 "$(psql -c "SELECT count(*) FROM audit_log WHERE action IS NULL OR at IS NULL OR tenant_id IS NULL")"
expect "every audit entry names its user, or the scheduled job, migration or demo seeding that wrote it" 0 "$(psql -c "SELECT count(*) FROM audit_log WHERE user_id IS NULL AND request_id NOT IN ('lifecycle', 'key-rotation', 'migrate') AND action <> 'demo_data_seeded'")"
echo "audit actions recorded: $(psql -c "SELECT string_agg(action || '=' || n, ' ' ORDER BY action) FROM (SELECT action, count(*) n FROM audit_log GROUP BY action) a")"
