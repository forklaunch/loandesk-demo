import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import { api, ApplicationDetail, date, DOC_TYPE_LABELS, money } from "../api";
import { RiskBadge, StatusBadge } from "./ApplicationList";

const DOC_STATUS: Record<string, string> = {
  pending: "Queued",
  processing: "Reading…",
  done: "Processed",
  failed: "Could not be read",
};

export default function ApplicationView({ id }: { id: string }) {
  const [data, setData] = useState<ApplicationDetail | null>(null);
  const [error, setError] = useState("");
  const [ssns, setSsns] = useState<Record<string, string>>({});

  const load = useCallback(
    () =>
      api<ApplicationDetail>("GET", `/applications/${id}`).then(
        (d) => (setData(d), setError("")),
        (err: Error) => setError(err.message),
      ),
    [id],
  );

  useEffect(() => {
    load();
  }, [load]);

  // While the worker is still reading a document, refresh every 2 seconds.
  const waiting = data?.documents.some((d) => d.status === "pending" || d.status === "processing");
  useEffect(() => {
    if (!waiting) return;
    const timer = setInterval(load, 2000);
    return () => clearInterval(timer);
  }, [waiting, load]);

  if (error && !data) return <div className="error">{error}</div>;
  if (!data) return null;
  const { application: a, owners, documents, permissions } = data;

  async function reveal(ownerId: string) {
    if (ssns[ownerId]) {
      const { [ownerId]: _, ...rest } = ssns;
      setSsns(rest);
      return;
    }
    try {
      const res = await api<{ ssn: string }>("POST", `/applications/${id}/owners/${ownerId}/reveal-ssn`);
      setSsns({ ...ssns, [ownerId]: res.ssn });
    } catch (err) {
      setError((err as Error).message);
    }
  }

  return (
    <>
      <div className="page-head">
        <div>
          <a href="#/" className="small">
            ← All applications
          </a>
          <h1>
            {a.legal_name} <span className="muted">{a.reference}</span>
          </h1>
        </div>
        <StatusBadge status={a.status} />
      </div>
      {error && <div className="error">{error}</div>}

      <div className="columns">
        <section className="card">
          <h2>Borrower</h2>
          <dl>
            <dt>EIN</dt>
            <dd>{a.ein}</dd>
            <dt>Address</dt>
            <dd>{a.address}</dd>
            <dt>Industry</dt>
            <dd>{a.industry}</dd>
            <dt>Requested</dt>
            <dd>{money(a.requested_amount)}</dd>
            <dt>Purpose</dt>
            <dd>{a.purpose}</dd>
            <dt>Relationship manager</dt>
            <dd>
              {a.created_by_name} · {date(a.created_at)}
            </dd>
          </dl>
        </section>

        <section className="card">
          <h2>
            Risk summary <RiskBadge flag={a.risk.flag} />
          </h2>
          {a.risk.missing.length > 0 && (
            <p className="muted">
              Waiting on {a.risk.missing.join(", ")} from uploaded documents before a risk flag can be shown.
            </p>
          )}
          <dl>
            <dt>Debt service coverage</dt>
            <dd>{a.risk.dscr !== null ? `${a.risk.dscr.toFixed(2)}x` : "—"}</dd>
            <dt>Leverage (debt / revenue)</dt>
            <dd>{a.risk.leverage !== null ? `${a.risk.leverage.toFixed(2)}x` : "—"}</dd>
            <dt>Est. annual debt service</dt>
            <dd>{money(a.risk.annual_debt_service)}</dd>
            <dt>Revenue</dt>
            <dd>{money(a.revenue)}</dd>
            <dt>Net income</dt>
            <dd>{money(a.net_income)}</dd>
            <dt>Existing debt</dt>
            <dd>{money(a.total_debt)}</dd>
          </dl>
          <p className="muted small">
            Figures are read automatically from uploaded documents; check them against the source. Debt service is
            estimated on existing debt plus this request at 8% over 10 years. Placeholder thresholds, not credit
            policy.
          </p>
        </section>
      </div>

      <section className="card">
        <h2>Owners</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>SSN</th>
              <th>Date of birth</th>
              <th className="num">Ownership</th>
            </tr>
          </thead>
          <tbody>
            {owners.map((o) => (
              <tr key={o.id} className="static">
                <td>{o.full_name}</td>
                <td>
                  <code>{ssns[o.id] ?? `•••-••-${o.ssn_last4}`}</code>{" "}
                  <button className="link" onClick={() => reveal(o.id)}>
                    {ssns[o.id] ? "Hide" : "Show"}
                  </button>
                </td>
                <td>{o.date_of_birth}</td>
                <td className="num">{o.ownership_percent}%</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="muted small">Showing a full SSN is recorded in the audit log.</p>
      </section>

      <section className="card">
        <h2>Documents</h2>
        {documents.length === 0 ? (
          <p className="muted">No documents uploaded yet.</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>File</th>
                <th>Type</th>
                <th>Status</th>
                <th className="num">Revenue</th>
                <th className="num">Net income</th>
                <th className="num">Total debt</th>
                <th>Uploaded</th>
              </tr>
            </thead>
            <tbody>
              {documents.map((d) => (
                <tr key={d.id} className="static">
                  <td>
                    <a href={`/api/documents/${d.id}/download`}>{d.filename}</a>
                    <div className="muted small">{Math.max(1, Math.round(d.size_bytes / 1024))} KB</div>
                  </td>
                  <td>{DOC_TYPE_LABELS[d.doc_type] ?? d.doc_type}</td>
                  <td>
                    <span className={`badge doc-${d.status}`}>{DOC_STATUS[d.status]}</span>
                    {d.status === "done" && d.revenue === null && d.net_income === null && d.total_debt === null && (
                      <div className="muted small">No key figures found</div>
                    )}
                  </td>
                  <td className="num">{money(d.revenue)}</td>
                  <td className="num">{money(d.net_income)}</td>
                  <td className="num">{money(d.total_debt)}</td>
                  <td>
                    {date(d.uploaded_at)}
                    <div className="muted small">{d.uploaded_by_name}</div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {permissions.can_upload && <UploadForm id={id} onDone={load} />}
      </section>

      <section className="card">
        <h2>Decision</h2>
        {a.status !== "submitted" ? (
          <>
            <p>
              <StatusBadge status={a.status} /> by {a.decided_by_name} on {date(a.decided_at)}
            </p>
            <blockquote>{a.decision_note}</blockquote>
          </>
        ) : permissions.can_decide ? (
          <DecisionForm id={id} onDone={load} />
        ) : (
          <p className="muted">
            {permissions.is_creator
              ? "Awaiting a credit officer. You cannot approve or decline an application you created."
              : "Awaiting a credit officer's decision."}
          </p>
        )}
      </section>
    </>
  );
}

function UploadForm({ id, onDone }: { id: string; onDone: () => void }) {
  const [docType, setDocType] = useState("financial_statement");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  async function submit(e: FormEvent) {
    e.preventDefault();
    const file = fileRef.current?.files?.[0];
    if (!file) return;
    setBusy(true);
    setError("");
    try {
      const body = new FormData();
      body.append("doc_type", docType);
      body.append("file", file);
      await api("POST", `/applications/${id}/documents`, body);
      if (fileRef.current) fileRef.current.value = "";
      onDone();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <form className="inline-form" onSubmit={submit}>
      <select value={docType} onChange={(e) => setDocType(e.target.value)} aria-label="Document type">
        {Object.entries(DOC_TYPE_LABELS).map(([value, label]) => (
          <option key={value} value={value}>
            {label}
          </option>
        ))}
      </select>
      <input ref={fileRef} type="file" accept="application/pdf,.pdf" required aria-label="PDF file" />
      <button className="primary" disabled={busy}>
        {busy ? "Uploading…" : "Upload PDF"}
      </button>
      {error && <div className="error">{error}</div>}
    </form>
  );
}

function DecisionForm({ id, onDone }: { id: string; onDone: () => void }) {
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function decide(decision: "approved" | "declined") {
    if (!note.trim()) {
      setError("Add a note explaining the decision.");
      return;
    }
    if (!window.confirm(`Mark this application as ${decision}? This cannot be undone.`)) return;
    setBusy(true);
    setError("");
    try {
      await api("POST", `/applications/${id}/decision`, { decision, note });
      onDone();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <label>
        Note
        <textarea value={note} onChange={(e) => setNote(e.target.value)} rows={3} maxLength={2000} />
      </label>
      {error && <div className="error">{error}</div>}
      <div className="actions">
        <button className="approve" disabled={busy} onClick={() => decide("approved")}>
          Approve
        </button>
        <button className="decline" disabled={busy} onClick={() => decide("declined")}>
          Decline
        </button>
      </div>
    </div>
  );
}
