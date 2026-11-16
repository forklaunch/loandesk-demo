import { useEffect, useState } from "react";
import { apiPage, Application, date, money, User } from "../api";
import { ErrorMessage, SearchIcon } from "../ui";

export function RiskBadge({ flag }: { flag: Application["risk"]["flag"] }) {
  if (!flag) return <span className="badge none">No data yet</span>;
  return <span className={`badge risk-${flag}`}>{flag[0].toUpperCase() + flag.slice(1)}</span>;
}

export function StatusBadge({ status }: { status: Application["status"] }) {
  return <span className={`badge status-${status}`}>{status[0].toUpperCase() + status.slice(1)}</span>;
}

export default function ApplicationList({ user }: { user: User }) {
  const [search, setSearch] = useState("");
  const [rows, setRows] = useState<Application[] | null>(null);
  const [next, setNext] = useState<string | null>(null);
  const [error, setError] = useState("");

  // Search text is sent in the request body, never in the URL: it is often an
  // EIN or a person's name. Lists come one page at a time.
  const fetchPage = (q: string, cursor?: string) =>
    q
      ? apiPage<Application>("POST", "/applications/search", { q, ...(cursor ? { cursor } : {}) })
      : apiPage<Application>("GET", "/applications" + (cursor ? `?cursor=${encodeURIComponent(cursor)}` : ""));

  useEffect(() => {
    // Small delay so we search when typing pauses, not on every key.
    let stale = false;
    const timer = setTimeout(() => {
      fetchPage(search.trim()).then(
        (page) => !stale && (setRows(page.items), setNext(page.next), setError("")),
        (err: Error) => !stale && setError(err.message),
      );
    }, 200);
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [search]);

  const isRm = user.role === "relationship_manager";

  async function loadMore() {
    if (!next) return;
    try {
      const page = await fetchPage(search.trim(), next);
      setRows([...(rows ?? []), ...page.items]);
      setNext(page.next);
    } catch (err) {
      setError((err as Error).message);
    }
  }

  return (
    <>
      <div className="page-head">
        <div>
          <h1>{isRm ? "My applications" : "Loan applications"}</h1>
          <p className="sub">
            {isRm
              ? "Applications you created, newest first."
              : "Every application submitted to the credit desk, newest first."}
          </p>
        </div>
        {isRm && (
          <a className="button primary" href="#/new">
            New application
          </a>
        )}
      </div>
      <div className="search">
        <SearchIcon />
        <input
          type="search"
          placeholder="Search borrower, EIN, owner, industry, purpose, reference…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          aria-label="Search applications"
        />
      </div>
      {error && <ErrorMessage>{error}</ErrorMessage>}
      <div className="card">
        {!rows && !error && (
          <div className="state" role="status">
            Loading applications…
          </div>
        )}
        {rows && rows.length === 0 && (
          <div className="state">
            <strong>{search.trim() ? "No applications match your search." : "No applications yet."}</strong>
            {search.trim()
              ? "Names and purpose match whole words or the first three letters of a word."
              : isRm
                ? "Create the first one with New application."
                : "Applications appear here once a relationship manager submits them."}
          </div>
        )}
        {rows && rows.length > 0 && (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Reference</th>
                  <th>Borrower</th>
                  <th>Industry</th>
                  <th className="num">Requested</th>
                  <th>Risk</th>
                  <th>Status</th>
                  {!isRm && <th>Relationship manager</th>}
                  <th>Created</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((a) => (
                  <tr key={a.id} className="link" onClick={() => (window.location.hash = `#/applications/${a.id}`)}>
                    <td>
                      <a className="ref" href={`#/applications/${a.id}`}>
                        {a.reference}
                      </a>
                    </td>
                    <td>
                      <div className="primary-text">{a.legal_name}</div>
                      <div className="secondary">EIN {a.ein}</div>
                    </td>
                    <td>{a.industry}</td>
                    <td className="num">{money(a.requested_amount)}</td>
                    <td>
                      <RiskBadge flag={a.risk.flag} />
                    </td>
                    <td>
                      <StatusBadge status={a.status} />
                    </td>
                    {!isRm && <td>{a.created_by_name}</td>}
                    <td className="nowrap">{date(a.created_at)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
      {next && (
        <div className="more">
          <button onClick={loadMore}>Load more</button>
        </div>
      )}
    </>
  );
}
