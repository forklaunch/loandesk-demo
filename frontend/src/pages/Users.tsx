import { FormEvent, useEffect, useState } from "react";
import { AccessReview, accessReviewCsv, api, apiPage, date, Role, ROLE_LABELS, User } from "../api";
import { ErrorMessage, Notice, Section } from "../ui";

const ROLES = Object.keys(ROLE_LABELS) as Role[];
const emptyForm = { full_name: "", email: "", password: "", role: "relationship_manager" as Role };

export default function Users({ me }: { me: User }) {
  const [users, setUsers] = useState<User[] | null>(null);
  const [form, setForm] = useState(emptyForm);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [review, setReview] = useState<AccessReview | null>(null);
  const [notice, setNotice] = useState("");

  // The API returns one page at a time; follow the cursor until the list is complete.
  const load = async () => {
    try {
      const all: User[] = [];
      let cursor: string | null = null;
      do {
        const page: { items: User[]; next: string | null } = await apiPage<User>(
          "GET",
          "/users?limit=100" + (cursor ? `&cursor=${encodeURIComponent(cursor)}` : ""),
        );
        all.push(...page.items);
        cursor = page.next;
      } while (cursor);
      setUsers(all);
      setReview(await api<AccessReview>("GET", "/access-review"));
    } catch (e) {
      setError((e as Error).message);
    }
  };
  useEffect(() => {
    load();
  }, []);

  async function add(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      await api("POST", "/users", form);
      setForm(emptyForm);
      await load();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  async function setRole(id: string, role: Role) {
    setError("");
    try {
      await api("PATCH", `/users/${id}`, { role });
    } catch (err) {
      setError((err as Error).message);
    }
    await load();
  }

  async function setDisabled(u: User, disabled: boolean) {
    setError("");
    if (
      disabled &&
      !window.confirm(`Disable ${u.full_name}? They are signed out and cannot sign in until re-enabled.`)
    ) {
      return;
    }
    try {
      await api("POST", `/users/${u.id}/${disabled ? "disable" : "enable"}`);
    } catch (err) {
      setError((err as Error).message);
    }
    await load();
  }

  async function recordReview() {
    setError("");
    setNotice("");
    try {
      const recorded = await api<{ accounts: number; stale_accounts: number }>("POST", "/access-review");
      setNotice(`Review recorded: ${recorded.accounts} accounts, ${recorded.stale_accounts} stale.`);
    } catch (err) {
      setError((err as Error).message);
    }
    await load();
  }

  function downloadReview() {
    if (!review) return;
    const url = URL.createObjectURL(new Blob([accessReviewCsv(review)], { type: "text/csv" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = `loandesk-access-review-${review.generated_at.slice(0, 10)}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  }

  const reviewed = new Map((review?.users ?? []).map((u) => [u.id, u]));

  return (
    <>
      <div className="page-head">
        <div>
          <h1>Users</h1>
          <p className="sub">Who can sign in, with which role. Admins manage accounts and never see applications.</p>
        </div>
      </div>
      {error && <ErrorMessage>{error}</ErrorMessage>}
      <div className="card">
        {!users && !error && (
          <div className="state" role="status">
            Loading users…
          </div>
        )}
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Email</th>
                <th>Role</th>
                <th>Added</th>
                <th>Last sign-in</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {(users ?? []).map((u) => (
                <tr key={u.id}>
                  <td className="primary-text">{u.full_name}</td>
                  <td>{u.email}</td>
                  <td>
                    {u.id === me.id ? (
                      <span title="You cannot change your own role">
                        {ROLE_LABELS[u.role]} <span className="muted">(you)</span>
                      </span>
                    ) : (
                      <select
                        value={u.role}
                        onChange={(e) => setRole(u.id, e.target.value as Role)}
                        aria-label={`Role for ${u.full_name}`}
                      >
                        {ROLES.map((r) => (
                          <option key={r} value={r}>
                            {ROLE_LABELS[r]}
                          </option>
                        ))}
                      </select>
                    )}
                  </td>
                  <td className="nowrap">{date(u.created_at ?? null)}</td>
                  <td className="nowrap">{u.last_login_at ? date(u.last_login_at) : "Never"}</td>
                  <td className="nowrap">
                    {u.disabled_at ? (
                      <span className="badge status-declined">Disabled</span>
                    ) : reviewed.get(u.id)?.stale ? (
                      <span
                        className="badge status-submitted"
                        title={`Unused for ${reviewed.get(u.id)?.days_since_last_use} days`}
                      >
                        Stale
                      </span>
                    ) : (
                      <span className="badge status-approved">Active</span>
                    )}{" "}
                    {u.id !== me.id && (
                      <button className="link" onClick={() => setDisabled(u, !u.disabled_at)}>
                        {u.disabled_at ? "Enable" : "Disable"}
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {review && (
        <Section
          title="Access review"
          aside={
            review.review_overdue ? (
              <span className="badge risk-medium">Review due</span>
            ) : review.last_review_at ? (
              <span className="badge status-approved">Up to date</span>
            ) : (
              <span className="badge">Not yet recorded</span>
            )
          }
        >
          <div className="card-body">
            {notice && <Notice>{notice}</Notice>}
            <p className={review.review_overdue ? undefined : "muted"}>
              {review.last_review_at
                ? `Last review recorded ${date(review.last_review_at)}.`
                : "No review has been recorded yet."}{" "}
              {review.review_overdue
                ? `A review is due (every ${review.review_interval_days} days).`
                : `The next one is due within ${review.review_interval_days} days of it.`}
            </p>
            <p className="muted">
              {review.users.length} accounts, {review.stale_accounts} unused for more than {review.stale_after_days}{" "}
              days. Go through the list above (roles, two-factor, last sign-in), disable what is no longer needed, then
              record the review. Unused accounts are disabled automatically after {review.stale_after_days} days.
            </p>
            <div className="actions">
              <button onClick={downloadReview}>Download as CSV</button>
              <button className="primary" onClick={recordReview}>
                Record that I reviewed access
              </button>
            </div>
          </div>
        </Section>
      )}

      <form onSubmit={add}>
        <Section title="Add a user">
          <div className="card-body">
            <div className="grid">
              <label>
                Full name
                <input
                  value={form.full_name}
                  onChange={(e) => setForm({ ...form, full_name: e.target.value })}
                  required
                />
              </label>
              <label>
                Email
                <input
                  type="email"
                  value={form.email}
                  onChange={(e) => setForm({ ...form, email: e.target.value })}
                  required
                />
              </label>
              <label>
                Role
                <select value={form.role} onChange={(e) => setForm({ ...form, role: e.target.value as Role })}>
                  {ROLES.map((r) => (
                    <option key={r} value={r}>
                      {ROLE_LABELS[r]}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Initial password (12+ characters)
                <input
                  type="password"
                  value={form.password}
                  onChange={(e) => setForm({ ...form, password: e.target.value })}
                  minLength={12}
                  autoComplete="new-password"
                  required
                />
              </label>
            </div>
            <div className="form-foot">
              <button className="primary" disabled={busy} aria-busy={busy}>
                Add user
              </button>
            </div>
          </div>
        </Section>
      </form>
    </>
  );
}
