#!/usr/bin/env python3
"""A small load test of the main read paths, against a running stack.

    docker compose up --build -d --wait
    python3 scripts/load-test.py                 # 20 s, 4 users, 8 requests/s each

Four signed-in users (two relationship managers, two credit officers) each
send a steady stream of requests: 75% the application list, 20% one
application with its owners and documents (decrypting its personal fields),
5% a search over encrypted text. It measures latency per request and fails if
the run misses the targets below, which are the objectives in docs/slo.md:

    - no response is a server error (5xx)
    - 95% of requests answer within 1 second
    - nothing is refused by the rate limits at this rate (no 429)
    - at least 90% of the intended requests were actually sent (the stack kept up)

The load is deliberately inside the per-user rate limit (600 requests a minute,
60 searches): the limit, not the hardware, caps what one user can ask for.
Standard library only. LOAD_SECONDS and LOAD_RATE change the run.
"""
import os
import random
import statistics
import sys
import threading
import time

sys.path.insert(0, os.path.dirname(__file__))
from e2e import BASE, Client  # noqa: E402

USERS = ["maya.chen@loandesk.demo", "daniel.ortiz@loandesk.demo", "priya.raman@loandesk.demo", "tom.becker@loandesk.demo"]
SECONDS = float(os.environ.get("LOAD_SECONDS", "20"))
RATE = float(os.environ.get("LOAD_RATE", "8"))  # requests per second, per user
P95_TARGET_MS = 1000


def worker(email, results):
    client = Client()
    status, _, _ = client.login(email)
    assert status == 200, f"{email} could not sign in ({status})"
    ids = [a["id"] for a in client.call("GET", "/api/applications?limit=20")[1]]
    rng = random.Random(email)
    start = time.monotonic()
    sent = 0
    while True:
        due = start + sent / RATE
        now = time.monotonic()
        if due - start >= SECONDS:
            break
        if due > now:
            time.sleep(due - now)
        roll = rng.random()
        began = time.monotonic()
        if roll < 0.75:
            kind, (status, _, _) = "list", client.call("GET", "/api/applications?limit=20")
        elif roll < 0.95:
            kind, (status, _, _) = "detail", client.call("GET", f"/api/applications/{rng.choice(ids)}")
        else:
            kind, (status, _, _) = "search", client.call("POST", "/api/applications/search", {"q": rng.choice(["dental", "brewing", "whitfield", "canning"])})
        results.append((kind, status, (time.monotonic() - began) * 1000))
        sent += 1


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * fraction))]


def main():
    results = []
    threads = [threading.Thread(target=worker, args=(email, results)) for email in USERS]
    began = time.monotonic()
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    elapsed = time.monotonic() - began
    intended = int(SECONDS * RATE) * len(USERS)
    latencies = [ms for _, _, ms in results]
    server_errors = sum(1 for _, status, _ in results if status >= 500)
    limited = sum(1 for _, status, _ in results if status == 429)
    other = sum(1 for _, status, _ in results if status not in (200, 429) and status < 500)
    p50, p95, p99 = statistics.median(latencies), percentile(latencies, 0.95), percentile(latencies, 0.99)
    print(f"target      {BASE}")
    print(f"load        {len(USERS)} users x {RATE:g} requests/s for {SECONDS:g} s")
    print(f"requests    {len(results)} of {intended} intended in {elapsed:.1f} s ({len(results) / elapsed:.1f} requests/s)")
    for kind in ("list", "detail", "search"):
        own = [ms for k, _, ms in results if k == kind]
        if own:
            print(f"  {kind:<8}  {len(own):>5} requests   p50 {statistics.median(own):6.1f} ms   p95 {percentile(own, 0.95):6.1f} ms")
    print(f"latency     p50 {p50:.1f} ms   p95 {p95:.1f} ms   p99 {p99:.1f} ms   max {max(latencies):.1f} ms")
    print(f"errors      {server_errors} server errors, {limited} rate-limited, {other} other")
    failures = []
    if server_errors:
        failures.append(f"{server_errors} server errors")
    if limited or other:
        failures.append(f"{limited} requests were rate-limited and {other} failed otherwise")
    if p95 > P95_TARGET_MS:
        failures.append(f"p95 {p95:.0f} ms is over the {P95_TARGET_MS} ms objective")
    if len(results) < 0.9 * intended:
        failures.append(f"only {len(results)} of {intended} requests were sent: the stack did not keep up")
    if failures:
        print("FAIL  " + "; ".join(failures))
        sys.exit(1)
    print(f"ok    {len(results) / elapsed:.0f} requests/s sustained with p95 {p95:.0f} ms (objective: {P95_TARGET_MS} ms), no errors")


if __name__ == "__main__":
    main()
