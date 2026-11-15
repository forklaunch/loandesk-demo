# 1. Move the backend onto ForkLaunch

Date: 2026-11-15. Status: accepted.

**Context.** The generated backend had hand-rolled routing, auth checks inside handlers, a
table used as a queue, and files on a local volume. The report card could only run its
language-independent checks.

**Decision.** One ForkLaunch Rust service (`api`) and one consumer-only leased-queue worker
(`extraction`), scaffolded with the CLI. Routes are typed contracts; roles are declared on the
contract and enforced by one policy. The existing cookie sessions are kept (resolved through a
ForkLaunch `EndpointPolicy`) rather than moving users to the IAM module, so nobody's login changes.
The worker and API share one database so a document and its job commit together.

**Consequences.** The front end keeps its endpoints except where a finding required a change
(upload, download, search, pagination: see CHANGELOG). Handlers return the framework's fixed
errors; the specific message the UI shows travels through `api/transport.rs`. Sign-in and
sign-out set the cookie through the same layer because contracts may not declare `Set-Cookie`.
