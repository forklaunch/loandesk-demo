# Access review

Who can use LoanDesk, with which role, is reviewed on a schedule, and unused access is removed
without waiting for the review.

| | Default | Setting |
| --- | --- | --- |
| Review interval | every 90 days | `ACCESS_REVIEW_INTERVAL_DAYS` |
| An account is stale when unused for | 90 days | `ACCESS_STALE_DAYS` |
| What happens to a stale account | disabled, sessions ended | `ACCESS_STALE_ACTION` (`disable` or `flag`) |

**These are demo defaults.** The person who owns access control has to confirm or change them
(`docs/needs-a-person.md`).

## Doing a review

1. Sign in as an admin and open **Users**. The table shows each account's role, last sign-in and
   status (active, stale, disabled). The same data, with second-factor status and open sessions,
   is `GET /api/access-review`; **Download as CSV** saves it for the record.
2. For each account: is the person still on the team, and is the role still right? Change the role
   or press **Disable**. Both take effect at once and end that account's sessions.
3. Press **Record that I reviewed access**. That writes an `access_review_recorded` entry to the
   append-only audit trail with your user id, the time, and the number of accounts, stale accounts
   and disabled accounts at that moment. It cannot be edited or back-dated.

A review is not recorded by the system on its own: an entry exists only because an admin pressed
the button after looking.

## Stale accounts

The worker's scheduled job (`src/extraction/lifecycle.rs`, hourly) disables accounts that have not
signed in within the stale period and ends their sessions; each gets a `user_disabled` audit entry
with detail `inactive` and no user id (the job did it). The inactivity count starts at the last
sign-in, or at creation or re-enabling when that is more recent. The most recently used admin is
never disabled when no other usable admin would remain; the job logs `stale_admin_kept` instead.

To give access back: **Enable** on the Users page. The count starts again from that moment.

## When it is overdue

The job logs `access_review_overdue` on every pass while no review has been recorded within the
interval, the `LoanDeskAccessReviewOverdue` alert raises a ticket, and the Users page says a review
is due.

## Removing access now

**Disable** on the Users page (or `POST /api/users/{id}/disable`). The account cannot sign in and
its sessions end in the same transaction. A role change does the same for the old role.
