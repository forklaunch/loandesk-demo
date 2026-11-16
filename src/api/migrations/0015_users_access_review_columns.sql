-- Access review and account lifecycle (docs/controls.md, "Access is reviewed and revoked").
--
-- Additive only: five columns, nullable or with a default. The version of the
-- API that is still running during a rolling deploy neither reads nor writes
-- them, so this migration is safe to apply before the new version starts
-- (expand step). Adding a column with a non-volatile default does not rewrite
-- the table: PostgreSQL stores the value once, in the catalog.
--
--   last_login_at        set at every successful sign-in
--   access_baseline_at   when counting inactivity starts for this account: the
--                        moment this column was added for existing accounts,
--                        creation for new ones, re-enabling for disabled ones.
--                        An account is stale when neither this nor
--                        last_login_at is newer than ACCESS_STALE_DAYS.
--   disabled_at          a disabled account cannot sign in and has no sessions
--   disabled_reason      'admin' (an admin disabled it) or 'inactive' (the
--                        scheduled job disabled it after ACCESS_STALE_DAYS)
--   password_changed_at  set when the user changes their own password
SET LOCAL lock_timeout = '5s';
ALTER TABLE users
    ADD COLUMN last_login_at       timestamptz,
    ADD COLUMN access_baseline_at  timestamptz NOT NULL DEFAULT now(),
    ADD COLUMN disabled_at         timestamptz,
    ADD COLUMN disabled_reason     text CHECK (disabled_reason IN ('admin', 'inactive')),
    ADD COLUMN password_changed_at timestamptz;
