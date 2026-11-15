#!/bin/sh
# Creates the LoanDesk database and its two roles. Idempotent.
#
#   loandesk_owner  owns the schema and runs migrations
#   loandesk_app    what the API and worker run as: no superuser, no BYPASSRLS,
#                   no CREATEROLE; only DML on the owner's tables
#
# Used by docker-compose.yaml with generated local passwords. A deployment
# provisions the same two roles through its own tooling (docs/deployment.md).
set -eu
s=/run/demo-secrets
export PGPASSWORD="$(cat $s/postgres_password)"
owner_password="$(cat $s/db_owner_password)"
app_password="$(cat $s/db_app_password)"
psql="psql -v ON_ERROR_STOP=1 -h postgresql -U postgres"

$psql -d postgres -v owner_password="$owner_password" -v app_password="$app_password" <<'SQL'
SELECT format('CREATE ROLE %I LOGIN', name) FROM unnest(ARRAY['loandesk_owner', 'loandesk_app']) AS name
 WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = name)\gexec
ALTER ROLE loandesk_owner PASSWORD :'owner_password' NOSUPERUSER NOCREATEROLE NOBYPASSRLS;
ALTER ROLE loandesk_app   PASSWORD :'app_password'   NOSUPERUSER NOCREATEROLE NOBYPASSRLS;
SELECT 'CREATE DATABASE loandesk OWNER loandesk_owner'
 WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'loandesk')\gexec
SQL

$psql -d loandesk <<'SQL'
ALTER SCHEMA public OWNER TO loandesk_owner;
REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO loandesk_app;
ALTER DEFAULT PRIVILEGES FOR ROLE loandesk_owner GRANT USAGE ON SCHEMAS TO loandesk_app;
ALTER DEFAULT PRIVILEGES FOR ROLE loandesk_owner GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO loandesk_app;
ALTER DEFAULT PRIVILEGES FOR ROLE loandesk_owner GRANT USAGE, SELECT ON SEQUENCES TO loandesk_app;
SELECT format('GRANT USAGE ON SCHEMA %I TO loandesk_app', nspname) FROM pg_namespace WHERE nspowner = 'loandesk_owner'::regrole\gexec
SELECT format('GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA %I TO loandesk_app', nspname) FROM pg_namespace WHERE nspowner = 'loandesk_owner'::regrole\gexec
SELECT format('GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA %I TO loandesk_app', nspname) FROM pg_namespace WHERE nspowner = 'loandesk_owner'::regrole\gexec
SQL
echo "database provisioned"
