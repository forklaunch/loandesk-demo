import { useEffect, useState } from "react";
import { api, Application, date, money, User } from "../api";

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
  const [error, setError] = useState("");

  useEffect(() => {
    // Small delay so we search when typing pauses, not on every key.
    let stale = false;
    const timer = setTimeout(() => {
      const q = search.trim();
      api<Application[]>("GET", "/applications" + (q ? `?q=${encodeURIComponent(q)}` : "")).then(
        (data) => !stale && (setRows(data), setError("")),
        (err: Error) => !stale && setError(err.message),
      );
    }, 200);
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [search]);

  const isRm = user.role === "relationship_manager";

  return (
    <>
      <div className="page-head">
        <h1>{isRm ? "My applications" : "Loan applications"}</h1>
        {isRm && (
          <a className="button primary" href="#/new">
            New application
          </a>
        )}
      </div>
      <input
        className="search"
        type="search"
        placeholder="Search borrower, EIN, owner, industry, purpose, reference…"
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        aria-label="Search applications"
      />
      {error && <div className="error">{error}</div>}
      {rows && rows.length === 0 && (
        <p className="muted">{search.trim() ? "No applications match your search." : "No applications yet."}</p>
      )}
      {rows && rows.length > 0 && (
        <div className="card table-wrap">
          <table>
            <thead>
              <tr>
                <th>Ref</th>
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
                <tr key={a.id} onClick={() => (window.location.hash = `#/applications/${a.id}`)}>
                  <td>
                    <a href={`#/applications/${a.id}`}>{a.reference}</a>
                  </td>
                  <td>
                    <strong>{a.legal_name}</strong>
                    <div className="muted small">EIN {a.ein}</div>
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
                  <td>{date(a.created_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </>
  );
}
