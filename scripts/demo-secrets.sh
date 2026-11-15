#!/bin/sh
# LOCAL DEMO ONLY.
#
# Generates random secrets for the local Docker Compose stack the first time it
# starts and keeps them in the `demo-secrets` volume. Nothing secret is stored
# in this repository, and no two checkouts share a key or password.
#
# A real deployment does not use this: it supplies the same settings from its
# secret manager (see docs/deployment.md), and the API refuses to start with
# APP_ENV=production while DEMO_SECRETS=true.
set -eu
umask 022
dir=/run/demo-secrets
hex() { od -An -tx1 -N"$1" /dev/urandom | tr -d ' \n'; }
make() { [ -s "$dir/$1" ] || { printf '%s' "$2" > "$dir/$1.tmp" && mv "$dir/$1.tmp" "$dir/$1"; }; }

make encryption_key_hex    "$(hex 32)"
make postgres_password     "$(hex 24)"
make db_owner_password     "$(hex 24)"
make db_app_password       "$(hex 24)"
make s3_access_key         "loandesk$(hex 6)"
make s3_secret_key         "$(hex 24)"
make storage_kms_key       "loandesk-demo:$(head -c 32 /dev/urandom | base64 | tr -d '\n')"
make worker_control_token  "$(hex 24)"
chmod 0444 "$dir"/*
echo "demo secrets ready"
