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
#
# Two more modes rehearse a key rotation on the local stack
# (docs/runbooks/key-rotation.md, scripts/verify-lifecycle.py):
#
#   docker compose run --rm demo-secrets rotate   # new active key; the old one stays readable
#   docker compose run --rm demo-secrets retire   # forget the old key
set -eu
umask 022
dir=/run/demo-secrets
hex() { od -An -tx1 -N"$1" /dev/urandom | tr -d ' \n'; }
make() { [ -s "$dir/$1" ] || { printf '%s' "$2" > "$dir/$1.tmp" && mv "$dir/$1.tmp" "$dir/$1"; }; }
put()  { printf '%s' "$2" > "$dir/$1.tmp" && chmod 0444 "$dir/$1.tmp" && mv -f "$dir/$1.tmp" "$dir/$1"; }

case "${1:-}" in
  rotate)
    if [ -s "$dir/encryption_previous_keys_json" ]; then
      echo "a previous key is still configured: finish that rotation (api --rotate-keys) and retire it first" >&2
      exit 1
    fi
    old_id="$(cat "$dir/encryption_key_id" 2>/dev/null || echo demo-1)"
    new_id="demo-$(( ${old_id#demo-} + 1 ))"
    put encryption_previous_keys_json "{\"$old_id\":\"$(cat "$dir/encryption_key_hex")\"}"
    put encryption_key_hex "$(hex 32)"
    put encryption_key_id "$new_id"
    echo "active key is now $new_id; $old_id stays readable until retired"
    exit 0
    ;;
  retire)
    rm -f "$dir/encryption_previous_keys_json"
    echo "previous key retired"
    exit 0
    ;;
  "") ;;
  *) echo "usage: demo-secrets [rotate|retire]" >&2; exit 2 ;;
esac

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
