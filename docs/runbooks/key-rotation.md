# Rotating the field-encryption key

One key (`ENCRYPTION_KEY_HEX`, named by `ENCRYPTION_KEY_ID`) protects every encrypted field, the
search tokens derived from them, and queued job payloads. Per-tenant and per-purpose keys are
derived from it.

**Schedule:** every 365 days (`ENCRYPTION_KEY_MAX_AGE_DAYS`), and at once if the key may have been
exposed. This is a demo default; the person who owns security has to confirm it
(`docs/needs-a-person.md`). `api --migrate` records the first time a key id is active
(`encryption_key_registered` in the audit trail); the worker's scheduled job logs
`encryption_key_rotation_due` when that is longer ago than the period, and the
`LoanDeskEncryptionKeyRotationDue` alert raises a ticket.

## Steps

1. **Add the new key.** Generate one (`openssl rand -hex 32`) in the secret manager. Set
   `ENCRYPTION_KEY_ID` to a new name and `ENCRYPTION_KEY_HEX` to the new key, and put the old one in
   `ENCRYPTION_PREVIOUS_KEYS_JSON` as `{"<old id>":"<old hex>"}`. Roll the API and the worker.
   From now on new values and search tokens are written under the new key; old rows still open,
   and search matches rows indexed under either key.
2. **Re-encrypt.** Run `api --rotate-keys` with the migration role (the same way `api --migrate`
   is run). It re-seals owners (name, SSN, date of birth), applications (purpose, decision note),
   document file names and two-factor secrets, and rebuilds each application's search tokens,
   200 rows per transaction. It can run while the service is up, can be interrupted and run again,
   and prints a report; it exits with an error unless `"remaining": 0`.
3. **Retire the old key.** Remove it from `ENCRYPTION_PREVIOUS_KEYS_JSON` and roll the API and the
   worker again. Wait until no queued or dead-lettered job from before step 1 is left
   (`extraction --dead-letters 100`; dead letters are purged after 30 days): their payloads were
   sealed under the old key.

Keep one previous key at a time: finish a rotation before starting the next.

## What records it

`protected_fields_rekeyed` (one audit entry per batch, with counts), `encryption_key_rotated` when
a run finishes with nothing left, and `encryption_key_registered` for the new key id.

## Rehearsing it

On the local stack: `python3 scripts/verify-lifecycle.py` performs all three steps with
`docker compose run --rm demo-secrets rotate` and `... retire`, and checks that every record opens
with the same contents afterwards, that no ciphertext or search token under the old key is left,
and that sign-in with a second factor, search, upload and extraction still work.
