#!/usr/bin/env python3
"""Checks, against the running Compose stack, the controls that only show over
time: retention, access review, password change, rate limits, the scheduled
job's lock, and an encryption key rotation from start to finish.

    docker compose up --build -d --wait
    python3 scripts/e2e.py
    python3 scripts/verify-lifecycle.py

It moves dates back in the database to make records old (as the PostgreSQL
superuser, the only one who can), runs the worker's scheduled pass by hand
(`extraction --lifecycle-once`), and rotates the demo stack's encryption key:
afterwards the stack runs on a new key (`demo-2` after the first run). It restarts the API and the
worker twice. Standard library only.
"""
import json
import os
import subprocess
import sys
import time
import urllib.request
import uuid

sys.path.insert(0, os.path.dirname(__file__))
import e2e  # noqa: E402  (the same HTTP client and helpers the end-to-end checks use)
from e2e import BASE, Client, check, new_application, signed_in, totp, upload  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), "..")
BUCKET = "loandesk-documents"
# What the scheduled job logged in the passes this script ran by hand.
JOB_OUTPUT = []


def compose(*args, stdin=None, ok=(0,)):
    result = subprocess.run(["docker", "compose", *args], cwd=ROOT, input=stdin, capture_output=True, text=True)
    if result.returncode not in ok:
        print(result.stdout[-2000:], result.stderr[-2000:])
        raise SystemExit(f"docker compose {' '.join(args)} failed ({result.returncode})")
    return result


def sql(statement):
    """Runs one statement as the PostgreSQL superuser and returns its output."""
    command = 'PGPASSWORD="$(cat /run/demo-secrets/postgres_password)" psql -v ON_ERROR_STOP=1 -U postgres -d loandesk -At -c "$1"'
    return compose("exec", "-T", "postgresql", "sh", "-c", command, "sh", statement).stdout.strip()


def stored(key):
    """Whether an object exists in the document store."""
    command = (
        'mc alias set s http://localhost:9000 "$(cat /run/demo-secrets/s3_access_key)" '
        '"$(cat /run/demo-secrets/s3_secret_key)" >/dev/null && mc stat "s/$1" >/dev/null 2>&1'
    )
    return compose("exec", "-T", "object-store", "sh", "-c", command, "sh", f"{BUCKET}/{key}", ok=(0, 1)).returncode == 0


def report_of(output, field):
    """The JSON report a command printed among its log lines."""
    for line in reversed(output.splitlines()):
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if isinstance(value, dict) and field in value:
            return value
    raise SystemExit(f"no report with `{field}` in:\n{output[-2000:]}")


def lifecycle_once():
    """One pass of the worker's scheduled job, in the worker's own container."""
    result = compose("exec", "-T", "extraction", "/bin/sh", "/usr/local/bin/demo-env", "/app/bin/extraction", "--lifecycle-once")
    JOB_OUTPUT.append(result.stdout + result.stderr)
    return report_of(result.stdout, "applications_purged")


def audit(admin, action, entity=None):
    """Recent audit entries of one action (newest first), optionally for one
    record. Reads back through the trail a page at a time and stops one page
    after the first match."""
    found, cursor = [], None
    for _ in range(30):
        status, page, headers = admin.call("GET", "/api/audit-log?limit=100" + (f"&cursor={cursor}" if cursor else ""))
        assert status == 200, (status, page)
        matched = [e for e in page if e["action"] == action and (entity is None or e["entity_id"] == entity)]
        cursor = headers.get("x-next-cursor")
        if (found and not matched) or not cursor:
            found += matched
            break
        found += matched
    return found


def wait_ready():
    deadline = time.time() + 90
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(BASE + "/api/readyz", timeout=5) as response:
                if response.status == 200:
                    return
        except Exception:
            pass
        time.sleep(1)
    raise SystemExit("the API did not become ready again")


def restart_backend():
    compose("restart", "api", "extraction")
    wait_ready()
    compose("up", "-d", "--wait", "--no-recreate", "api", "extraction")


def wait_document(client, application, document):
    deadline = time.time() + 60
    while time.time() < deadline:
        detail = client.call("GET", f"/api/applications/{application}")[1]
        row = next(d for d in detail["documents"] if d["id"] == document)
        if row["status"] in ("done", "failed"):
            return row
        time.sleep(1)
    raise SystemExit("the document was not processed in time")


def main():
    with open(e2e.SAMPLE, "rb") as handle:
        pdf = handle.read()
    maya = signed_in("maya.chen@loandesk.demo")
    priya = signed_in("priya.raman@loandesk.demo")
    admin = signed_in("admin@loandesk.demo")
    tenant = sql("SELECT DISTINCT tenant_id FROM users")

    # ---------------------------------------------------------------- retention
    tag = uuid.uuid4().hex[:8]
    status, old, _ = new_application(maya, f"Retention Old {tag} LLC")
    check(status == 201, "an application is created to be aged past its retention period")
    status, doc = upload(maya, old["id"], pdf, "old-statement.pdf")
    check(status == 200 and wait_document(maya, old["id"], doc["id"])["status"] == "done", "it has a processed document")
    status, _, _ = priya.call("POST", f"/api/applications/{old['id']}/decision", {"decision": "declined", "note": "Retention rehearsal."})
    check(status == 200, "it is decided")
    status, kept, _ = new_application(maya, f"Retention Kept {tag} LLC")
    status, kept_doc = upload(maya, kept["id"], pdf, "kept-statement.pdf")
    check(status == 200, "a second, recent application with a document is left alone")
    # An upload grant that is used but never confirmed: the file is in storage, the row says "uploading".
    status, grant, _ = maya.call("POST", f"/api/applications/{kept['id']}/documents", {"doc_type": "tax_return", "filename": "abandoned.pdf", "size_bytes": len(pdf)})
    body, content_type = e2e.multipart(grant["fields"], "document.pdf", pdf)
    urllib.request.urlopen(urllib.request.Request(BASE + grant["url"], data=body, method="POST", headers={"Content-Type": content_type}), timeout=60).read()

    old_key = f"tenants/{tenant}/applications/{old['id']}/{doc['id']}.pdf"
    kept_key = f"tenants/{tenant}/applications/{kept['id']}/{kept_doc['id']}.pdf"
    abandoned_key = f"tenants/{tenant}/applications/{kept['id']}/{grant['id']}.pdf"
    check(stored(old_key) and stored(kept_key) and stored(abandoned_key), "all three files are in object storage")

    first = lifecycle_once()
    check(first["applications_purged"] == 0 and first["failed_steps"] == 0, "nothing is purged while every record is within its period")

    # Age the decided application one day past the default (2,555 days) and the abandoned upload past 24 hours.
    sql(f"UPDATE applications SET created_at = now() - interval '2600 days', decided_at = now() - interval '2556 days' WHERE id = '{old['id']}'")
    sql(f"UPDATE documents SET uploaded_at = now() - interval '25 hours' WHERE id = '{grant['id']}'")

    # The scheduled job runs once, not once per instance: while another session
    # holds its lock, a pass does nothing.
    holder = subprocess.Popen(
        ["docker", "compose", "exec", "-T", "postgresql", "sh", "-c",
         'PGPASSWORD="$(cat /run/demo-secrets/postgres_password)" psql -U postgres -d loandesk -At -c "SELECT pg_advisory_lock(4107, 2)" -c "SELECT pg_sleep(8)"'],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    time.sleep(2)
    blocked = lifecycle_once()
    holder.wait()
    check(blocked["skipped"] is True and blocked["applications_purged"] == 0, "a pass skips while another instance holds the job's lock")
    check(maya.call("GET", f"/api/applications/{old['id']}")[0] == 200, "the skipped pass deleted nothing")

    purge = lifecycle_once()
    check(purge["skipped"] is False and purge["applications_purged"] == 1 and purge["documents_purged"] == 1, "the scheduled job deletes the application past its retention period")
    check(purge["unfinished_uploads_removed"] == 1 and purge["files_deleted"] == 2 and purge["failed_steps"] == 0, "and the upload that was never confirmed, with both stored files")
    check(maya.call("GET", f"/api/applications/{old['id']}")[0] == 404 and priya.call("GET", f"/api/applications/{old['id']}")[0] == 404, "the purged application is gone for everyone (404)")
    counts = sql(f"SELECT (SELECT count(*) FROM applications WHERE id = '{old['id']}') + (SELECT count(*) FROM owners WHERE application_id = '{old['id']}') + (SELECT count(*) FROM documents WHERE application_id = '{old['id']}' OR id = '{grant['id']}')")
    check(counts == "0", "its owners and document rows went with it")
    check(not stored(old_key) and not stored(abandoned_key), "deleting the record deleted its stored files")
    check(stored(kept_key) and maya.call("GET", f"/api/applications/{kept['id']}")[0] == 200, "the recent application and its file are untouched")
    entries = audit(admin, "application_purged", old["id"])
    check(
        len(entries) == 1 and entries[0]["user_id"] is None and entries[0]["detail"] == "retention=decided days=2555 documents=1" and entries[0]["at"],
        "the purge is in the audit trail: what, which record, when, and under which rule",
    )
    check(len(audit(admin, "document_upload_expired", grant["id"])) == 1, "so is the removed upload")
    again = lifecycle_once()
    check(again["applications_purged"] == 0 and again["files_deleted"] == 0, "running the job again changes nothing")

    # ------------------------------------------------------------ access review
    status, review, _ = admin.call("GET", "/api/access-review")
    by_email = {u["email"]: u for u in review["users"]}
    check(
        status == 200 and by_email["maya.chen@loandesk.demo"]["role"] == "relationship_manager" and by_email["maya.chen@loandesk.demo"]["last_login_at"] and by_email["admin@loandesk.demo"]["open_sessions"] >= 1,
        "an admin gets the access review: every account with role, last sign-in and open sessions",
    )
    check(review["stale_after_days"] == 90 and review["review_interval_days"] == 90 and review["truncated"] is False, "it states the stale period and the review interval in force")
    check(maya.call("GET", "/api/access-review")[0] == 403 and priya.call("POST", "/api/access-review")[0] == 403, "only admins can read or record an access review (403)")
    status, recorded, _ = admin.call("POST", "/api/access-review")
    check(status == 201 and recorded["accounts"] == len(review["users"]), "the admin records that the review was done")
    me = by_email["admin@loandesk.demo"]["id"]
    status, review, _ = admin.call("GET", "/api/access-review")
    check(review["last_review_at"] and review["last_review_by"] == me and review["review_overdue"] is False, "the review shows when it was last done, and by whom")
    entry = audit(admin, "access_review_recorded")[0]
    check(entry["user_id"] == me and entry["detail"].startswith("accounts="), "the recorded review is an audit entry: who, when, how many accounts")

    email = f"stale-{tag}@loandesk.demo"
    password = "first-" + uuid.uuid4().hex
    status, account, _ = admin.call("POST", "/api/users", {"email": email, "full_name": "Stale Tester", "role": "credit_officer", "password": password})
    stale_session = Client()
    check(status == 201 and stale_session.login(email, password)[0] == 200, "a new account signs in")
    sql(f"UPDATE users SET last_login_at = now() - interval '91 days', access_baseline_at = now() - interval '200 days' WHERE id = '{account['id']}'")
    listed = next(u for u in admin.call("GET", "/api/access-review")[1]["users"] if u["id"] == account["id"])
    check(listed["stale"] is True and listed["days_since_last_use"] == 91 and listed["status"] == "active", "an account unused for 91 days is flagged as stale in the review")
    swept = lifecycle_once()
    check(swept["stale_accounts"] == 1 and swept["accounts_disabled"] == 1, "the scheduled job disables the stale account")
    check(stale_session.call("GET", "/api/auth/me")[0] == 401, "its open session ends at once (401)")
    status, body, _ = Client().login(email, password)
    wrong = Client().login(email, "not-the-password-1")[1]["error"]
    check(status == 401 and body["error"] == wrong, "a disabled account cannot sign in, and the answer does not say why")
    entry = audit(admin, "user_disabled", account["id"])[0]
    check(entry["user_id"] is None and entry["detail"] == "inactive", "the automatic disabling is in the audit trail")
    listed = next(u for u in admin.call("GET", "/api/access-review")[1]["users"] if u["id"] == account["id"])
    check(listed["status"] == "disabled" and listed["disabled_reason"] == "inactive" and listed["stale"] is False, "the review shows it as disabled for inactivity")
    status, _, _ = admin.call("POST", f"/api/users/{account['id']}/enable")
    user = Client()
    check(status == 200 and user.login(email, password)[0] == 200, "an admin re-enables it and it signs in again")
    check(lifecycle_once()["accounts_disabled"] == 0, "re-enabling restarts the inactivity count")
    status, _, _ = admin.call("POST", f"/api/users/{account['id']}/disable")
    check(status == 200 and user.call("GET", "/api/auth/me")[0] == 401, "an admin disables an account; its session ends at once (401)")
    status, body, _ = admin.call("POST", f"/api/users/{me}/disable")
    check(status == 403 and body["error"] == "You cannot disable your own account", "an admin cannot disable their own account (403)")
    check(maya.call("POST", f"/api/users/{account['id']}/enable")[0] == 403, "a relationship manager cannot enable or disable accounts (403)")
    admin.call("POST", f"/api/users/{account['id']}/enable")

    # ---------------------------------------------------------- password change
    here, elsewhere = Client(), Client()
    check(here.login(email, password)[0] == 200 and elsewhere.login(email, password)[0] == 200, "the user is signed in on two devices")
    new_password = "second-" + uuid.uuid4().hex
    status, body, _ = here.call("POST", "/api/auth/password", {"current_password": "not-the-password-1", "new_password": new_password})
    check(status == 400 and body["error"] == "The current password is not correct", "a password change with the wrong current password is refused (400)")
    status, body, _ = here.call("POST", "/api/auth/password", {"current_password": password, "new_password": "short"})
    check(status == 400, "a new password under 12 characters is refused (400)")
    status, body, _ = here.call("POST", "/api/auth/password", {"current_password": password, "new_password": new_password})
    check(status == 200 and body == {"ok": True}, "the user changes their password")
    check(elsewhere.call("GET", "/api/auth/me")[0] == 401, "the password change ends the session on the other device (401)")
    check(here.call("GET", "/api/auth/me")[0] == 200, "the device that changed it stays signed in, on a new session")
    check(Client().login(email, password)[0] == 401 and Client().login(email, new_password)[0] == 200, "the old password no longer works; the new one does")
    check(len(audit(admin, "password_changed", account["id"])) == 1 and len(audit(admin, "password_change_refused", account["id"])) == 1, "both attempts are in the audit trail")

    # -------------------------------------------------------------- rate limits
    statuses = [here.call("POST", "/api/applications/search", {"q": "dental"})[0] for _ in range(70)]
    check(statuses[0] == 200 and 429 in statuses and set(statuses) <= {200, 429}, "the expensive-operation limit refuses a burst of searches from one user (429)")
    status, body, _ = here.call("POST", "/api/applications/search", {"q": "dental"})
    check(status == 429 and "Too many requests" in body["error"], "with a message that says to wait")
    check(here.call("GET", "/api/applications?limit=1")[0] == 200, "ordinary requests from the same user are not affected")
    check(priya.call("POST", "/api/applications/search", {"q": "dental"})[0] == 200, "nor is anyone else")
    bogus = Client()
    bogus.call("GET", "/api/auth/me", headers={"Cookie": "loandesk_session=" + "0" * 64})
    logs = compose("logs", "--no-color", "--since", "15m", "api", "extraction").stdout + "".join(JOB_OUTPUT)
    for event in ("rate_limited", "invalid_session", "login_failed", "login_refused_disabled", "permission_denied", "password_changed", "password_change_refused", "user_disabled", "user_enabled", "access_review_recorded", "stale_accounts_disabled", "retention_purge", "lifecycle_pass"):
        check(f'"{event}"' in logs or f"event={event}" in logs, f"the `{event}` event is in the log, for the alert rules to count")

    # ------------------------------------------------------------- key rotation
    status, mfa_user, _ = admin.call("POST", "/api/users", {"email": f"mfa-{tag}@loandesk.demo", "full_name": "Rotation Tester", "role": "credit_officer", "password": password})
    mfa = Client()
    mfa.login(mfa_user["email"], password)
    secret = mfa.call("POST", "/api/auth/mfa/enrol")[1]["secret"]
    check(mfa.call("POST", "/api/auth/mfa/confirm", {"code": totp(secret)})[0] == 200, "a user with two-factor sign-in exists before the rotation")
    before = maya.call("GET", f"/api/applications/{kept['id']}")[1]
    owner = before["owners"][0]
    ssn = maya.call("POST", f"/api/applications/{kept['id']}/owners/{owner['id']}/reveal-ssn")[1]["ssn"]
    old_id = sql(f"SELECT split_part(purpose_ciphertext, '.', 2) FROM applications WHERE id = '{kept['id']}'")
    expected = f"demo-{int(old_id.split('-')[1]) + 1}"

    compose("run", "--rm", "--no-deps", "demo-secrets", "rotate")
    restart_backend()
    maya, admin, priya = (signed_in(e) for e in ("maya.chen@loandesk.demo", "admin@loandesk.demo", "priya.raman@loandesk.demo"))
    during = maya.call("GET", f"/api/applications/{kept['id']}")[1]
    check(during["application"]["purpose"] == before["application"]["purpose"] and during["owners"] == before["owners"], "with the new key active and the old one kept for reads, existing records still open")
    check(any(a["id"] == kept["id"] for a in e2e.find(maya, "yarborough")) and e2e.find(maya, "whitfield"), "search on encrypted names still finds rows indexed under the old key")
    status, fresh, _ = new_application(maya, f"Rotation New {tag} LLC")
    new_id = sql(f"SELECT split_part(purpose_ciphertext, '.', 2) FROM applications WHERE id = '{fresh['id']}'")
    check(status == 201 and new_id != old_id and new_id == expected, f"new records are written under the new key ({new_id}, was {old_id})")
    pending = sql(f"SELECT count(*) FROM applications WHERE purpose_ciphertext LIKE 'v3.{old_id}.%'")
    check(int(pending) > 0, "rows under the old key are waiting to be re-encrypted")

    rotated = report_of(compose("run", "--rm", "--no-deps", "api-migrate", "/app/bin/api", "--rotate-keys").stdout, "remaining")
    check(rotated["remaining"] == 0 and rotated["active_key_id"] == expected and rotated["applications"] >= int(pending) and rotated["owners"] > 0 and rotated["documents"] > 0 and rotated["mfa_secrets"] >= 1, "`api --rotate-keys` re-encrypts every row under the new key")
    left = sql(
        f"SELECT (SELECT count(*) FROM applications WHERE purpose_ciphertext LIKE 'v3.{old_id}.%' OR decision_note_ciphertext LIKE 'v3.{old_id}.%')"
        f" + (SELECT count(*) FROM owners WHERE full_name_ciphertext LIKE 'v3.{old_id}.%' OR date_of_birth_ciphertext LIKE 'v3.{old_id}.%')"
        f" + (SELECT count(*) FROM documents WHERE filename_ciphertext LIKE 'v3.{old_id}.%')"
        f" + (SELECT count(*) FROM user_mfa WHERE secret_ciphertext LIKE 'v3.{old_id}.%')"
        f" + (SELECT count(*) FROM applications a WHERE EXISTS (SELECT 1 FROM unnest(a.search_tokens) t WHERE t NOT LIKE 'bi1.{expected}.%'))"
    )
    check(left == "0", "no ciphertext and no search token under the old key is left in the database")
    again = report_of(compose("run", "--rm", "--no-deps", "api-migrate", "/app/bin/api", "--rotate-keys").stdout, "remaining")
    check(again["applications"] == 0 and again["owners"] == 0 and again["remaining"] == 0, "running the rotation again changes nothing")

    compose("run", "--rm", "--no-deps", "demo-secrets", "retire")
    restart_backend()
    maya, admin = signed_in("maya.chen@loandesk.demo"), signed_in("admin@loandesk.demo")
    after = maya.call("GET", f"/api/applications/{kept['id']}")[1]
    check(after["application"]["purpose"] == before["application"]["purpose"] and after["owners"] == before["owners"] and after["documents"][0]["filename"] == "kept-statement.pdf", "with the old key retired, every record still opens with the same contents")
    check(maya.call("POST", f"/api/applications/{kept['id']}/owners/{owner['id']}/reveal-ssn")[1]["ssn"] == ssn, "the full SSN is the same after re-encryption")
    check(any(a["id"] == kept["id"] for a in e2e.find(maya, "yarborough")) and e2e.find(maya, "whitfield") and e2e.find(maya, "canning"), "search on encrypted names and purpose works on the rebuilt index")
    check(Client().login(mfa_user["email"], password, totp(secret))[0] == 200, "two-factor sign-in still works: the stored secret was re-encrypted too")
    status, doc = upload(maya, fresh["id"], pdf, "after-rotation.pdf")
    check(status == 200 and wait_document(maya, fresh["id"], doc["id"])["status"] == "done", "uploads and extraction work on the new key")
    check(len(audit(admin, "encryption_key_rotated")) >= 1 and any(e["detail"] == expected for e in audit(admin, "encryption_key_registered")), "the rotation and the new key's first use are in the audit trail")

    print(f"\n{e2e.passed} checks passed against {BASE}")


if __name__ == "__main__":
    main()
