#!/bin/sh
# Checks how the running Compose stack behaves as a service, after
# `python3 scripts/e2e.py` has exercised it:
#   1. nothing a user entered (names, SSNs, dates of birth, free text, file
#      names, passwords) is in any container's log;
#   2. a browser error report is logged, without its message;
#   3. a background job carries the trace id of the request that queued it;
#   4. readiness fails while a dependency is down, liveness does not;
#   5. SIGTERM drains and is logged, for the API and the worker.
# It stops and starts the object store, the API and the worker.
set -eu
BASE="${LOANDESK_URL:-http://127.0.0.1:41080}"
ok()   { echo "ok    $1"; }
fail() { echo "FAIL  $1"; exit 1; }
status() { curl -s -o /dev/null -m 10 -w '%{http_code}' "$BASE$1" || true; }
wait_for() { # path, wanted status, seconds
  i=0
  while [ "$i" -lt "$3" ]; do
    [ "$(status "$1")" = "$2" ] && return 0
    i=$((i + 1)); sleep 1
  done
  return 1
}
logs() { docker compose logs --no-color --no-log-prefix "$@" 2>&1; }

# --- 1. no personal data, free text or credential in any log -----------------
# Seeded owners (src/api/seed.rs), what scripts/e2e.py enters, and the passwords.
all_logs="$(docker compose logs --no-color 2>&1)"
[ "$(printf '%s\n' "$all_logs" | grep -c 'http_request')" -gt 20 ] \
  || fail "the logs hold too few request lines: run scripts/e2e.py first"
leaked=0
while IFS= read -r value; do
  [ -n "$value" ] || continue
  if printf '%s\n' "$all_logs" | grep -F -i -q -- "$value"; then
    echo "FAIL  found in a container log: $value"
    printf '%s\n' "$all_logs" | grep -F -i -m 3 -- "$value" | cut -c1-240
    leaked=1
  fi
done <<'VALUES'
Whitfield
Sofia Ortega
Kowalski
Trevor Banks
Lindqvist
Naomi Feldman
Grant Okafor
Walter Brandt
Brandt-Reyes
Isabel Navarro
Yarborough
900114321
900-11-4321
900228765
901335502
902447790
902447815
903561248
904672933
905783016
906894127
907915238
908126349
909237450
900551234
900-55-1234
1978-04-12
1980-02-03
zebracorn
canning line
Covered by cash flow
cedar-ridge-financial-statement.pdf
loandesk-demo-2026
wrong-password-123
X-Amz-Signature
E2E Tester
VALUES
[ "$leaked" = 0 ] || exit 1
ok "no owner name, SSN, date of birth, free text, file name, password or signed link is in any container log"

# --- 2. browser errors are reported ------------------------------------------
line="$(logs api | grep '"browser_error"\|event=browser_error\|browser_error' | tail -1)"
[ -n "$line" ] || fail "no browser_error line in the API log (scripts/e2e.py reports one)"
case "$line" in *TypeError*index-e2e.js:40:1234*) ;; *) fail "the browser error line lacks its class or stack frame: $line" ;; esac
case "$line" in *"must be 9 digits"*) fail "the browser error line contains the error message" ;; esac
ok "a browser error is logged with its class and stack frames, without its message"

# --- 3. trace context crosses the queue --------------------------------------
trace="4bf92f3577b34da6a3ce929d0e0e4736"   # sent by scripts/e2e.py with the upload
logs extraction | grep "job_completed" | grep -q "$trace" \
  || fail "no completed job in the worker log carries trace id $trace of the request that queued it"
ok "the worker's job log carries the trace id of the request that queued the job"

# --- 4. readiness follows the dependencies, liveness does not ----------------
[ "$(status /api/readyz)" = 200 ] || fail "the API is not ready to begin with"
docker compose stop object-store >/dev/null 2>&1
wait_for /api/readyz 503 20 || fail "readiness stayed 200 with the object store stopped"
[ "$(status /api/healthz)" = 200 ] || fail "liveness failed because a dependency was down"
ok "readiness returns 503 while the object store is down; liveness stays 200"
docker compose start object-store >/dev/null 2>&1
wait_for /api/readyz 200 60 || fail "readiness did not recover after the object store came back"
ok "readiness recovers when the dependency returns"

# --- 5. graceful, logged shutdown --------------------------------------------
for service in api extraction; do
  docker compose stop "$service" >/dev/null 2>&1
  out="$(logs "$service")"
  printf '%s\n' "$out" | grep -q 'shutdown_started' || fail "$service did not log shutdown_started on SIGTERM"
  printf '%s\n' "$out" | grep -q 'shutdown_complete' || fail "$service did not log shutdown_complete"
  code="$(docker inspect -f '{{.State.ExitCode}}' "$(docker compose ps -a -q "$service")")"
  [ "$code" = 0 ] || fail "$service exited with $code after SIGTERM (killed instead of drained?)"
  ok "$service drains on SIGTERM, logs shutdown_started and shutdown_complete, and exits 0"
  docker compose start "$service" >/dev/null 2>&1
done
wait_for /api/readyz 200 90 || fail "the API did not come back"
ok "the stack is serving again"
