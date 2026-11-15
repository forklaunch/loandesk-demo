# 3. Encrypt personal fields; search them by keyed token

Date: 2026-11-15. Status: accepted.

**Context.** Owner names, dates of birth and free text were stored in clear and searched with
`ILIKE`. ForkLaunch persisted models only allow ciphertext for classified fields.

**Decision.** Seal them as `ProtectedField`s. Keep search by storing HMAC tokens of each word and
its prefixes (3 to 12 characters) in `applications.search_tokens` (GIN index).

**Consequences.** Search on owner names and purpose matches words and word prefixes, not arbitrary
substrings. Tokens reveal equality of word prefixes within a tenant, not the words. After a key
rotation, tokens must be rebuilt before searches under the new key find old rows (not yet scripted).
Every decryption is written to the audit trail, so viewing an application adds several audit rows.
