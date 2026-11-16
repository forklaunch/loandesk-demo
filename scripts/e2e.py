#!/usr/bin/env python3
"""End-to-end checks against a running LoanDesk stack (real HTTP, real
PostgreSQL, real object storage, real worker). Standard library only.

    docker compose up --build -d --wait
    python3 scripts/e2e.py                      # http://127.0.0.1:41080

Needs the demo data (SEED_DEMO_DATA=true, the Compose default). Covers the main
flows and the access rules: who is refused, with which status.
"""
import base64
import hashlib
import hmac
import http.cookiejar
import json
import os
import struct
import sys
import time
import urllib.error
import urllib.request
import uuid

BASE = os.environ.get("LOANDESK_URL", "http://127.0.0.1:41080").rstrip("/")
PASSWORD = os.environ.get("DEMO_PASSWORD", "loandesk-demo-2026")
SAMPLE = os.path.join(os.path.dirname(__file__), "..", "sample-docs", "cedar-ridge-financial-statement.pdf")
# A fixed W3C trace context sent with document uploads, to follow a request into its job.
TRACE_ID = "4bf92f3577b34da6a3ce929d0e0e4736"
TRACEPARENT = f"00-{TRACE_ID}-00f067aa0ba902b7-01"
passed = 0


def check(condition, label):
    global passed
    if not condition:
        print(f"FAIL  {label}")
        sys.exit(1)
    passed += 1
    print(f"ok    {label}")


class Client:
    """One browser session: its own cookie jar."""

    def __init__(self):
        self.opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))

    def call(self, method, path, body=None, headers=None, csrf=True):
        url = path if path.startswith("http") else BASE + path
        data = None
        all_headers = {"Origin": BASE} if method != "GET" else {}
        if csrf:
            all_headers["X-Requested-With"] = "loandesk"
        if body is not None:
            data = json.dumps(body).encode()
            all_headers["Content-Type"] = "application/json"
        all_headers.update(headers or {})
        request = urllib.request.Request(url, data=data, method=method, headers=all_headers)
        try:
            with self.opener.open(request, timeout=30) as response:
                raw = response.read()
                return response.status, _json(raw), response.headers
        except urllib.error.HTTPError as error:
            return error.code, _json(error.read()), error.headers

    def login(self, email, password=PASSWORD, totp=None):
        body = {"email": email, "password": password}
        if totp:
            body["totp"] = totp
        return self.call("POST", "/api/auth/login", body)


def _json(raw):
    try:
        return json.loads(raw)
    except ValueError:
        return raw


def signed_in(email):
    client = Client()
    status, user, _ = client.login(email)
    check(status == 200 and user["email"] == email, f"{email} signs in")
    return client


def multipart(fields, filename, content):
    boundary = uuid.uuid4().hex
    parts = []
    for name, value in fields.items():
        parts.append(f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"\r\n\r\n{value}\r\n'.encode())
    parts.append(
        f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="{filename}"\r\n'
        f"Content-Type: application/pdf\r\n\r\n".encode() + content + b"\r\n"
    )
    parts.append(f"--{boundary}--\r\n".encode())
    return b"".join(parts), f"multipart/form-data; boundary={boundary}"


def upload(client, application, content, filename="statement.pdf", declared=None):
    status, grant, _ = client.call(
        "POST",
        f"/api/applications/{application}/documents",
        {"doc_type": "financial_statement", "filename": filename, "size_bytes": declared or len(content)},
    )
    if status != 201:
        return status, grant
    body, content_type = multipart(grant["fields"], "document.pdf", content)
    url = grant["url"] if grant["url"].startswith("http") else BASE + grant["url"]
    request = urllib.request.Request(url, data=body, method="POST", headers={"Content-Type": content_type})
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            stored = response.status
    except urllib.error.HTTPError as error:
        return error.code, {"error": "storage refused the upload", "stage": "storage"}
    assert stored in (200, 201, 204), stored
    # The request that queues the job carries a trace context; the worker's log
    # lines for that job must carry the same trace id (scripts/verify-operations.sh).
    status, accepted, _ = client.call("POST", f"/api/documents/{grant['id']}/complete", headers={"traceparent": TRACEPARENT})
    return status, accepted if status != 200 else {**accepted, "id": grant["id"]}


def totp(secret, at=None):
    key = base64.b32decode(secret + "=" * (-len(secret) % 8))
    counter = struct.pack(">Q", int((at or time.time()) // 30))
    digest = hmac.new(key, counter, hashlib.sha1).digest()
    offset = digest[19] & 0x0F
    return f"{(struct.unpack('>I', digest[offset:offset + 4])[0] & 0x7FFFFFFF) % 1_000_000:06d}"


def find(client, text):
    status, rows, _ = client.call("POST", "/api/applications/search", {"q": text})
    assert status == 200, (status, rows)
    return rows


def new_application(client, name, key=None):
    body = {
        "legal_name": name,
        "ein": "98-7654321",
        "address": "1 Test Street, Springfield, IL 62701",
        "industry": "Testing",
        "requested_amount": 125000,
        "purpose": "Working capital for the zebracorn expansion.",
        "owners": [
            {"full_name": "Quentin Yarborough", "ssn": "900-55-1234", "date_of_birth": "1980-02-03", "ownership_percent": 100}
        ],
    }
    return client.call("POST", "/api/applications", body, {"Idempotency-Key": key} if key else None)


def main():
    anonymous = Client()

    # --- health, transport, anonymous access --------------------------------
    status, live, headers = anonymous.call("GET", "/api/healthz")
    check(status == 200 and live["status"] == "ok" and live["checks"] == [], "liveness answers without checking dependencies")
    status, ready, _ = anonymous.call("GET", "/api/readyz")
    check(
        status == 200 and {c["name"]: c["status"] for c in ready["checks"]} == {"database": "ok", "object_store": "ok"},
        "readiness reports the database and the object store",
    )
    check(headers.get("x-request-id") and "default-src 'none'" in headers.get("content-security-policy", ""), "responses carry a request id and a content security policy")
    check(headers.get("x-content-type-options") == "nosniff" and headers.get("x-frame-options") == "DENY", "nosniff and frame denial are set")
    status, body, _ = anonymous.call("GET", "/api/applications")
    check(status == 401 and body["error"] == "Not signed in", "the application list refuses an anonymous caller (401)")
    status, _, _ = anonymous.call("GET", "/api/users")
    check(status == 401, "the user list refuses an anonymous caller (401)")
    status, body, _ = anonymous.login("maya.chen@loandesk.demo", "wrong-password-123")
    wrong = body["error"]
    status2, body2, _ = anonymous.login("nobody@loandesk.demo", "wrong-password-123")
    check(status == 401 and status2 == 401 and wrong == body2["error"], "a wrong password and an unknown email get the same answer")
    status, _, _ = anonymous.call("POST", "/api/auth/login", {"email": "maya.chen@loandesk.demo", "password": PASSWORD}, csrf=False)
    check(status == 403, "state-changing requests without the CSRF header are refused (403)")
    status, _, _ = anonymous.call("POST", "/api/auth/login", {"email": "maya.chen@loandesk.demo", "password": PASSWORD}, {"Origin": "https://evil.example"})
    check(status == 403, "requests from another origin are refused (403)")

    maya = signed_in("maya.chen@loandesk.demo")
    daniel = signed_in("daniel.ortiz@loandesk.demo")
    priya = signed_in("priya.raman@loandesk.demo")
    admin = signed_in("admin@loandesk.demo")

    # --- visibility by role --------------------------------------------------
    status, mine, _ = maya.call("GET", "/api/applications")
    check(status == 200 and len(mine) >= 4 and all(a["created_by_name"] == "Maya Chen" for a in mine), "a relationship manager sees only their own applications")
    status, everything, _ = priya.call("GET", "/api/applications")
    check(status == 200 and len(everything) >= 7, "a credit officer sees every application")
    status, body, _ = admin.call("GET", "/api/applications")
    check(status == 403 and "Admins manage users" in body["error"], "an admin is refused borrower data (403)")
    status, body, _ = maya.call("GET", "/api/users")
    check(status == 403 and body["error"] == "Only admins can manage users", "a relationship manager is refused user management (403)")
    daniels = daniel.call("GET", "/api/applications")[1]
    status, _, _ = maya.call("GET", f"/api/applications/{daniels[0]['id']}")
    check(status == 404, "another manager's application looks like it does not exist (404)")

    # --- pagination ----------------------------------------------------------
    status, page1, headers = priya.call("GET", "/api/applications?limit=3")
    cursor = headers.get("x-next-cursor")
    check(status == 200 and len(page1) == 3 and cursor, "lists come one page at a time with a cursor")
    status, page2, _ = priya.call("GET", f"/api/applications?limit=3&cursor={cursor}")
    check(status == 200 and not {a["id"] for a in page1} & {a["id"] for a in page2}, "the next page continues without repeating rows")
    status, _, _ = priya.call("GET", "/api/applications?limit=1000")
    check(status == 400, "an oversized page request is refused (400)")

    # --- search (body, not URL) ---------------------------------------------
    check([a["legal_name"] for a in find(maya, "dental")] == ["Ortega Family Dental PLLC"], "search finds a borrower by name")
    check([a["legal_name"] for a in find(maya, "07-4410382")] == ["Harborview Brewing Company LLC"], "search finds a borrower by EIN")
    check([a["legal_name"] for a in find(maya, "whitfield")] == ["Harborview Brewing Company LLC"], "search finds an application by an owner's (encrypted) name")
    check([a["legal_name"] for a in find(maya, "canning")] == ["Harborview Brewing Company LLC"], "search finds an application by a word in its (encrypted) purpose")
    check(find(maya, "kowalski") == [], "search does not reach applications the caller cannot see")
    status, unfiltered, _ = maya.call("GET", "/api/applications?q=whitfield")
    check(status == 200 and len(unfiltered) == len(mine), "search text placed in the URL is ignored: there is no query-string search")

    # --- create, idempotency -------------------------------------------------
    key = str(uuid.uuid4())
    name = f"E2E Widgets {key[:8]} LLC"
    status, created, _ = new_application(maya, name, key)
    check(status == 201 and created["reference"].startswith("LD-"), "a relationship manager creates an application")
    status, again, _ = new_application(maya, name, key)
    check(status == 201 and again == created, "retrying with the same Idempotency-Key returns the same application")
    check(len(find(maya, name)) == 1, "the retry did not create a duplicate")
    status, body, _ = new_application(priya, "Not allowed LLC")
    check(status == 403 and "relationship managers" in body["error"], "a credit officer cannot create an application (403)")
    status, body, _ = maya.call("POST", "/api/applications", {"legal_name": "x", "ein": "12", "address": "a", "industry": "i", "requested_amount": 1, "purpose": "p", "owners": []})
    check(status == 400 and body["error"] == "EIN must be 9 digits, like 12-3456789", "validation errors keep their specific message (400)")
    check(len(find(maya, "zebracorn")) >= 1 and len(find(maya, "yarbor")) >= 1, "new applications are searchable by purpose and owner-name prefix")

    # --- strict input: unknown fields, privileged fields, array size -----------
    valid = {
        "legal_name": "Strict Input LLC", "ein": "98-7654321", "address": "1 Test Street", "industry": "Testing",
        "requested_amount": 1000, "purpose": "Testing.",
        "owners": [{"full_name": "Quentin Yarborough", "ssn": "900-55-1234", "date_of_birth": "1980-02-03", "ownership_percent": 1}],
    }
    for field, value in (("status", "approved"), ("created_by", str(uuid.uuid4())), ("revenue", 9_000_000), ("tenant_id", str(uuid.uuid4()))):
        status, _, _ = maya.call("POST", "/api/applications", {**valid, field: value})
        check(status == 400, f"a client cannot set `{field}` when creating an application: unknown fields are refused (400)")
    status, _, _ = maya.call("POST", "/api/applications", {**valid, "owners": [{**valid["owners"][0], "is_verified": True}]})
    check(status == 400, "an unknown field inside a nested object is refused too (400)")
    status, body, _ = maya.call("POST", "/api/applications", {**valid, "owners": valid["owners"] * 21})
    check(status == 400 and body["error"] == "Add between 1 and 20 owners", "an application with more than 20 owners is refused (400)")
    status, _, _ = maya.call("POST", "/api/applications", {**valid, "legal_name": "x" * 401})
    check(status == 400, "a string over its length limit is refused (400)")
    check(find(maya, "Strict Input") == [], "none of the refused requests created anything")

    # --- SSN handling --------------------------------------------------------
    status, detail, _ = maya.call("GET", f"/api/applications/{created['id']}")
    raw = json.dumps(detail)
    owner = detail["owners"][0]
    check(status == 200 and owner["ssn_last4"] == "1234" and owner["full_name"] == "Quentin Yarborough", "the application view shows the owner and the SSN's last four digits")
    check("900551234" not in raw and "900-55-1234" not in raw and "ssn" not in {k for o in detail["owners"] for k in o if k != "ssn_last4"}, "the application view never contains a full SSN")
    status, export, _ = maya.call("GET", f"/api/applications/{created['id']}/export")
    check(status == 200 and "900551234" not in json.dumps(export) and export["owners"][0]["ssn_last4"] == "1234", "the machine-readable export excludes full SSNs")
    status, revealed, _ = maya.call("POST", f"/api/applications/{created['id']}/owners/{owner['id']}/reveal-ssn")
    check(status == 200 and revealed == {"ssn": "900-55-1234"}, "the audited reveal endpoint returns the full SSN to an authorised user")
    status, _, _ = daniel.call("POST", f"/api/applications/{created['id']}/owners/{owner['id']}/reveal-ssn")
    check(status == 404, "another manager cannot reveal that SSN (404)")
    status, _, _ = admin.call("POST", f"/api/applications/{created['id']}/owners/{owner['id']}/reveal-ssn")
    check(status == 403, "an admin cannot reveal an SSN (403)")

    # --- audit trail: who, what, which record, when ---------------------------
    status, trail, _ = admin.call("GET", "/api/audit-log?limit=100")
    status2, first, headers = admin.call("GET", "/api/audit-log?limit=2")
    older = admin.call("GET", f"/api/audit-log?limit=2&cursor={headers.get('x-next-cursor')}")[1]
    check(
        status == 200 and status2 == 200 and len(first) == 2 and len(older) == 2 and older[0]["id"] < first[1]["id"] < first[0]["id"],
        "an admin reads the audit trail, newest first, one page at a time",
    )
    maya_id = maya.call("GET", "/api/auth/me")[1]["id"]
    reveal = next(e for e in trail if e["action"] == "ssn_revealed")
    check(
        reveal["user_id"] == maya_id and reveal["entity_id"] == owner["id"] and reveal["at"] and reveal["request_id"],
        "revealing an SSN left an audit entry: who, what, which owner, when",
    )
    reads = [e for e in trail if e["action"] == "protected_field_read" and e["user_id"] == maya_id]
    check(
        any(e["entity_id"] == owner["id"] and e["detail"].startswith("Owner.ssn_ciphertext") for e in reads)
        and any(e["entity_id"] == created["id"] and e["detail"] == "Application.purpose_ciphertext: view-application" for e in reads),
        "opening an application left an audit entry for each personal field read, with the reason",
    )
    check(any(e["action"] == "application_exported" and e["entity_id"] == created["id"] for e in trail), "so did the export")
    check(any(e["action"] == "application_created" and e["entity_id"] == created["id"] and e["user_id"] == maya_id for e in trail), "and creating the application")
    check("900551234" not in json.dumps(trail) and "Yarborough" not in json.dumps(trail), "audit entries hold identifiers and action names, never the values")
    status, _, _ = maya.call("GET", "/api/audit-log")
    check(status == 403, "only admins can read the audit trail (403)")

    # --- upload, extraction, download ---------------------------------------
    with open(SAMPLE, "rb") as handle:
        pdf = handle.read()
    status, doc = upload(maya, created["id"], pdf, "cedar-ridge-financial-statement.pdf")
    check(status == 200 and doc["status"] == "pending", "a PDF is uploaded straight to object storage and queued")
    status, repeat, _ = maya.call("POST", f"/api/documents/{doc['id']}/complete")
    check(status == 200, "confirming the same upload twice is harmless")
    deadline = time.time() + 60
    document = None
    while time.time() < deadline:
        detail = maya.call("GET", f"/api/applications/{created['id']}")[1]
        document = next(d for d in detail["documents"] if d["id"] == doc["id"])
        if document["status"] in ("done", "failed"):
            break
        time.sleep(1)
    check(document["status"] == "done" and document["revenue"] and document["net_income"] and document["total_debt"], "the worker extracts revenue, net income and total debt")
    application = detail["application"]
    check(application["revenue"] == document["revenue"] and application["risk"]["flag"] in ("low", "medium", "high"), f"the risk summary is computed from the figures ({application['risk']['flag']})")
    check(len([d for d in detail["documents"] if d["id"] == doc["id"]]) == 1 and document["filename"] == "cedar-ridge-financial-statement.pdf", "the document is listed once, under its original name")
    status, grant, _ = priya.call("POST", f"/api/documents/{doc['id']}/download-url")
    with urllib.request.urlopen(BASE + grant["url"] if grant["url"].startswith("/") else grant["url"], timeout=30) as response:
        downloaded = response.read()
    check(status == 200 and downloaded == pdf, "a credit officer downloads the stored PDF through a short-lived link")
    status, _, _ = daniel.call("POST", f"/api/documents/{doc['id']}/download-url")
    check(status == 404, "another manager cannot download it (404)")
    status, _, _ = admin.call("POST", f"/api/documents/{doc['id']}/download-url")
    check(status == 403, "an admin cannot download it (403)")
    status, body = upload(maya, created["id"], b"x", declared=21 * 1024 * 1024)
    check(status == 400 and "too large" in body["error"], "a file over 20 MB is refused before upload (400)")
    status, body = upload(maya, created["id"], b"GIF89a not a pdf at all")
    check(status == 400 and body["error"] == "Only PDF files can be uploaded", "a file that is not a PDF is refused and removed (400)")
    status, body = upload(maya, created["id"], b"%PDF-" + os.urandom(20 * 1024 * 1024 + 512 * 1024), declared=1024)
    check(status in (400, 413) and body.get("stage") == "storage", "object storage itself refuses a body over 20 MB, whatever size was declared")
    status, body = upload(priya, created["id"], pdf)
    check(status == 403, "a credit officer cannot upload documents (403)")

    # --- decisions -----------------------------------------------------------
    decision = {"decision": "approved", "note": "Covered by cash flow."}
    status, body, _ = maya.call("POST", f"/api/applications/{created['id']}/decision", decision)
    check(status == 403 and "credit officers" in body["error"], "a relationship manager cannot approve anything (403)")
    status, body, _ = admin.call("POST", f"/api/applications/{created['id']}/decision", decision)
    check(status == 403, "an admin cannot approve (403)")

    # A manager who later becomes a credit officer still cannot decide what they created.
    users = admin.call("GET", "/api/users?limit=100")[1]
    maya_user = next(u for u in users if u["email"] == "maya.chen@loandesk.demo")
    status, _, _ = admin.call("PATCH", f"/api/users/{maya_user['id']}", {"role": "credit_officer"})
    check(status == 200, "an admin changes a user's role")
    try:
        status, _, _ = maya.call("GET", "/api/auth/me")
        check(status == 401, "the role change ends that user's open sessions immediately (401)")
        maya_as_officer = signed_in("maya.chen@loandesk.demo")
        status, body, _ = maya_as_officer.call("POST", f"/api/applications/{created['id']}/decision", decision)
        check(status == 403 and body["error"] == "You cannot approve or decline an application you created", "self-approval is refused even for a credit officer (403)")
    finally:
        admin.call("PATCH", f"/api/users/{maya_user['id']}", {"role": "relationship_manager"})
    maya = signed_in("maya.chen@loandesk.demo")
    me = next(u for u in users if u["email"] == "admin@loandesk.demo")
    status, body, _ = admin.call("PATCH", f"/api/users/{me['id']}", {"role": "credit_officer"})
    check(status == 403 and body["error"] == "You cannot change your own role", "an admin cannot change their own role (403)")

    status, body, _ = priya.call("POST", f"/api/applications/{created['id']}/decision", decision)
    check(status == 200 and body == {"status": "approved"}, "a credit officer approves another person's application")
    status, body, _ = priya.call("POST", f"/api/applications/{created['id']}/decision", {"decision": "declined", "note": "Changed my mind."})
    check(status == 409 and body["error"] == "This application has already been decided", "a decided application cannot be decided again (409)")
    detail = maya.call("GET", f"/api/applications/{created['id']}")[1]
    check(detail["application"]["status"] == "approved" and detail["application"]["decision_note"] == "Covered by cash flow." and detail["application"]["decided_by_name"] == "Priya Raman", "the decision, note and decider are shown")
    check(detail["permissions"] == {"can_upload": False, "can_decide": False, "is_creator": True}, "a decided application is closed to uploads")

    # --- user management, two-factor ----------------------------------------
    email = f"e2e-{uuid.uuid4().hex[:8]}@loandesk.demo"
    new_password = "e2e-" + uuid.uuid4().hex
    status, user, _ = admin.call("POST", "/api/users", {"email": email, "full_name": "E2E Tester", "role": "credit_officer", "password": new_password})
    check(status == 201 and user["email"] == email and "password" not in json.dumps(user), "an admin adds a user")
    status, body, _ = admin.call("POST", "/api/users", {"email": email, "full_name": "E2E Tester", "role": "credit_officer", "password": new_password})
    check(status == 409, "a second user with the same email is refused (409)")
    tester = Client()
    check(tester.login(email, new_password)[0] == 200, "the new user signs in")
    status, enrolment, _ = tester.call("POST", "/api/auth/mfa/enrol")
    check(status == 200 and len(enrolment["secret"]) >= 16, "the user starts two-factor enrolment")
    status, body, _ = tester.call("POST", "/api/auth/mfa/confirm", {"code": "000000"})
    check(status == 400, "a wrong code does not turn two-factor on (400)")
    status, _, _ = tester.call("POST", "/api/auth/mfa/confirm", {"code": totp(enrolment["secret"])})
    check(status == 200 and tester.call("GET", "/api/auth/me")[1]["mfa_enabled"] is True, "a correct code turns two-factor sign-in on")
    second = Client()
    status, body, _ = second.login(email, new_password)
    check(status == 401 and body["error"] == wrong, "password alone no longer signs in, with the same generic answer (401)")
    check(second.login(email, new_password, totp(enrolment["secret"]))[0] == 200, "password plus authenticator code signs in")

    # --- browser error reports ------------------------------------------------
    report = {
        "kind": "unhandledrejection",
        "class": "TypeError",
        "screen": f"#/applications/{created['id']}",
        # A browser stack starts with the message, which can quote what a user
        # typed. Only the frames may be kept (checked in verify-operations.sh).
        "stack": "TypeError: SSN for Quentin Yarborough must be 9 digits\n    at submit (http://127.0.0.1:41080/assets/index-e2e.js:40:1234)",
        "release": "e2e",
    }
    status, body, _ = maya.call("POST", "/api/client-errors", report)
    check(status == 200 and body == {"ok": True}, "a signed-in browser can report an error it caught")
    status, _, _ = Client().call("POST", "/api/client-errors", report)
    check(status == 401, "an anonymous caller cannot write error reports (401)")
    status, _, _ = maya.call("POST", "/api/client-errors", {**report, "message": "typed text"})
    check(status == 400, "an error report with an unknown field (such as a message) is refused (400)")

    # --- sign-out ------------------------------------------------------------
    status, _, _ = tester.call("POST", "/api/auth/logout")
    check(status == 200 and tester.call("GET", "/api/auth/me")[0] == 401, "signing out ends the session (401 afterwards)")

    print(f"\n{passed} checks passed against {BASE}")


if __name__ == "__main__":
    main()
