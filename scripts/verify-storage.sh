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
expect "no legacy plaintext remains in owners" 0 "$(psql -c "SELECT count(*) FROM owners WHERE full_name IS NOT NULL OR ssn_encrypted IS NOT NULL OR ssn_last4 IS NOT NULL OR date_of_birth IS NOT NULL")"
expect "purpose, decision notes and file names are ciphertext" 0 "$(psql -c "SELECT (SELECT count(*) FROM applications WHERE purpose IS NOT NULL OR decision_note IS NOT NULL OR purpose_ciphertext !~ '^(v3|pd1)\.') + (SELECT count(*) FROM documents WHERE filename IS NOT NULL OR filename_ciphertext !~ '^(v3|pd1)\.')")"
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
echo "audit actions recorded: $(psql -c "SELECT string_agg(action || '=' || n, ' ' ORDER BY action) FROM (SELECT action, count(*) n FROM audit_log GROUP BY action) a")"
