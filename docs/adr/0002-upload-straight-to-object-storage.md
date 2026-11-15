# 2. Upload documents straight to object storage

Date: 2026-11-15. Status: accepted.

**Context.** Uploads were buffered in API memory and written to one instance's disk. ForkLaunch
contract bodies are capped (2 MiB per request, 1 MiB per multipart field); documents are up to 20 MB.

**Decision.** The API issues a presigned POST grant (fixed key, `application/pdf`, 20 MB,
5 minutes); the browser sends the file to storage; the API then verifies size and signature and
queues the job. Downloads use a 60-second presigned GET. Locally nginx proxies `/storage/` to
MinIO so everything stays same-origin.

**Consequences.** Upload is two API calls plus one storage call. A grant that is never used
leaves an `uploading` row that is cleared after an hour. Browser uploads cannot attach an object
tag, so the classification is recorded on the document row.
