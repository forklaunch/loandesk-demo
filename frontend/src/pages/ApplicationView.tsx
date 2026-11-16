import { DragEvent, FormEvent, useCallback, useEffect, useRef, useState } from "react";
import {
  api,
  ApplicationDetail,
  date,
  DOC_TYPE_LABELS,
  downloadDocument,
  money,
  uploadDocument,
  uploadLimitHint,
} from "../api";
import { RiskBadge, StatusBadge } from "./ApplicationList";
import { BackIcon, ErrorMessage, Section, UploadIcon } from "../ui";

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

  if (error && !data) return <ErrorMessage>{error}</ErrorMessage>;
  if (!data) {
    return (
      <div className="card">
        <div className="state" role="status">
          Loading application…
        </div>
      </div>
    );
  }
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

  const ratio = (value: number | null) => (value !== null ? `${value.toFixed(2)}x` : "—");
  const tiles: [string, string][] = [
    ["Requested", money(a.requested_amount)],
    ["Revenue", money(a.revenue)],
    ["Net income", money(a.net_income)],
    ["Existing debt", money(a.total_debt)],
    ["Debt service coverage", ratio(a.risk.dscr)],
    ["Leverage (debt / revenue)", ratio(a.risk.leverage)],
  ];

  return (
    <>
      <a href="#/" className="back">
        <BackIcon /> All applications
      </a>
      <div className="page-head">
        <div>
          <div className="record-title">
            <h1>{a.legal_name}</h1>
            <StatusBadge status={a.status} />
          </div>
          <div className="record-meta">
            <span className="ref">{a.reference}</span>
            <span>{a.industry}</span>
            <span>
              Requested <strong className="nowrap">{money(a.requested_amount)}</strong>
            </span>
            <span>
              {a.created_by_name} · {date(a.created_at)}
            </span>
          </div>
        </div>
      </div>
      {error && <ErrorMessage>{error}</ErrorMessage>}

      <Section
        title="Figures and risk"
        aside={<RiskBadge flag={a.risk.flag} />}
        foot={
          <>
            Figures are read automatically from uploaded documents; check them against the source. Debt service is
            estimated on existing debt plus this request at 8% over 10 years. Placeholder thresholds, not credit policy.
          </>
        }
      >
        <div className="card-body">
          <div className="tiles">
            {tiles.map(([label, value]) => (
              <div className="tile" key={label}>
                <div className="label">{label}</div>
                <div className={value === "—" ? "value empty" : "value"}>{value}</div>
              </div>
            ))}
          </div>
          <p className="muted">
            Est. annual debt service <strong className="nowrap">{money(a.risk.annual_debt_service)}</strong>
            {a.risk.missing.length > 0 && (
              <>
                {" · "}Waiting on {a.risk.missing.join(", ")} from uploaded documents before a risk flag can be shown.
              </>
            )}
          </p>
        </div>
      </Section>

      <div className="columns">
        <Section title="Borrower">
          <div className="card-body">
            <dl className="fields">
              <dt>Legal name</dt>
              <dd>{a.legal_name}</dd>
              <dt>EIN</dt>
              <dd>{a.ein}</dd>
              <dt>Address</dt>
              <dd>{a.address}</dd>
              <dt>Industry</dt>
              <dd>{a.industry}</dd>
            </dl>
          </div>
        </Section>
        <Section title="Request">
          <div className="card-body">
            <dl className="fields">
              <dt>Requested</dt>
              <dd className="nowrap">{money(a.requested_amount)}</dd>
              <dt>Purpose</dt>
              <dd>{a.purpose}</dd>
              <dt>Relationship manager</dt>
              <dd>
                {a.created_by_name} · {date(a.created_at)}
              </dd>
            </dl>
          </div>
        </Section>
      </div>

      <Section title="Owners" foot="Showing a full SSN is recorded in the audit log.">
        <div className="table-wrap">
          <table className="compact">
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
                <tr key={o.id}>
                  <td className="primary-text">{o.full_name}</td>
                  <td className="nowrap">
                    <code>{ssns[o.id] ?? `•••-••-${o.ssn_last4}`}</code>{" "}
                    <button className="link" onClick={() => reveal(o.id)}>
                      {ssns[o.id] ? "Hide" : "Show"}
                    </button>
                  </td>
                  <td className="nowrap">{o.date_of_birth}</td>
                  <td className="num">{o.ownership_percent}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>

      <Section title="Documents" aside={<span className="muted small">{documents.length} on file</span>}>
        {documents.length === 0 ? (
          <div className="state">
            <strong>No documents uploaded yet.</strong>
            Revenue, net income and existing debt are read from the financial statement, tax return and bank statement.
          </div>
        ) : (
          <div className="table-wrap">
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
                  <tr key={d.id}>
                    <td>
                      <a
                        href="#download"
                        onClick={(e) => {
                          e.preventDefault();
                          downloadDocument(d.id, d.filename).catch((err: Error) => setError(err.message));
                        }}
                      >
                        {d.filename}
                      </a>
                      <div className="secondary">{Math.max(1, Math.round(d.size_bytes / 1024))} KB</div>
                    </td>
                    <td className="nowrap">{DOC_TYPE_LABELS[d.doc_type] ?? d.doc_type}</td>
                    <td>
                      <span className={`badge doc-${d.status}`}>{DOC_STATUS[d.status]}</span>
                      {d.status === "done" && d.revenue === null && d.net_income === null && d.total_debt === null && (
                        <div className="secondary">No key figures found</div>
                      )}
                    </td>
                    <td className="num">{money(d.revenue)}</td>
                    <td className="num">{money(d.net_income)}</td>
                    <td className="num">{money(d.total_debt)}</td>
                    <td>
                      <div className="nowrap">{date(d.uploaded_at)}</div>
                      <div className="secondary">{d.uploaded_by_name}</div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {permissions.can_upload && (
          <div className="card-body">
            <UploadForm id={id} onDone={load} />
          </div>
        )}
      </Section>

      <Section title="Decision">
        <div className="card-body">
          {a.status !== "submitted" ? (
            <>
              <p>
                <StatusBadge status={a.status} /> by <strong>{a.decided_by_name}</strong> on {date(a.decided_at)}
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
        </div>
      </Section>
    </>
  );
}

function UploadForm({ id, onDone }: { id: string; onDone: () => void }) {
  const [docType, setDocType] = useState("financial_statement");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [over, setOver] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  async function submit(e: FormEvent) {
    e.preventDefault();
    const file = fileRef.current?.files?.[0];
    if (!file) return;
    setBusy(true);
    setError("");
    try {
      await uploadDocument(id, docType, file);
      if (fileRef.current) fileRef.current.value = "";
      onDone();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  // A file dropped on the area goes into the same file field the button reads.
  function drop(e: DragEvent<HTMLFormElement>) {
    e.preventDefault();
    setOver(false);
    if (fileRef.current && e.dataTransfer.files.length > 0) {
      fileRef.current.files = e.dataTransfer.files;
    }
  }

  return (
    <form
      className={over ? "dropzone over" : "dropzone"}
      onSubmit={submit}
      onDragOver={(e) => {
        e.preventDefault();
        setOver(true);
      }}
      onDragLeave={() => setOver(false)}
      onDrop={drop}
    >
      <div className="drop-icon">
        <UploadIcon />
      </div>
      <div>
        <div className="drop-title">Add a document</div>
        <div className="muted small">Drop a file here or choose one. {uploadLimitHint()}.</div>
      </div>
      <div className="drop-controls">
        <select value={docType} onChange={(e) => setDocType(e.target.value)} aria-label="Document type">
          {Object.entries(DOC_TYPE_LABELS).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
        <input ref={fileRef} type="file" accept="application/pdf,.pdf" required aria-label="PDF file" />
        <button className="primary" disabled={busy} aria-busy={busy}>
          {busy ? "Uploading…" : "Upload PDF"}
        </button>
      </div>
      {error && <ErrorMessage>{error}</ErrorMessage>}
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
    <div className="decision">
      <label>
        <span>
          Decision note <span className="hint">(required; kept with the application)</span>
        </span>
        <textarea value={note} onChange={(e) => setNote(e.target.value)} rows={3} maxLength={2000} />
      </label>
      {error && <ErrorMessage>{error}</ErrorMessage>}
      <div className="actions split">
        <button className="decline" disabled={busy} onClick={() => decide("declined")}>
          Decline
        </button>
        <button className="approve" disabled={busy} onClick={() => decide("approved")}>
          Approve
        </button>
      </div>
      <p className="muted small">A decision cannot be undone. You are asked to confirm.</p>
    </div>
  );
}
