import { FormEvent, useState } from "react";
import { api } from "../api";
import { BackIcon, ErrorMessage, Section } from "../ui";

interface OwnerForm {
  full_name: string;
  ssn: string;
  date_of_birth: string;
  ownership_percent: string;
}

const emptyOwner: OwnerForm = { full_name: "", ssn: "", date_of_birth: "", ownership_percent: "" };

export default function NewApplication() {
  const [form, setForm] = useState({
    legal_name: "",
    ein: "",
    address: "",
    industry: "",
    requested_amount: "",
    purpose: "",
  });
  const [owners, setOwners] = useState<OwnerForm[]>([{ ...emptyOwner }]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  // One key per form: if the request is retried (double click, flaky network),
  // the API returns the application it already created instead of a duplicate.
  const [idempotencyKey] = useState(() => crypto.randomUUID());

  const set = (key: keyof typeof form) => (e: { target: { value: string } }) =>
    setForm({ ...form, [key]: e.target.value });
  const setOwner = (i: number, key: keyof OwnerForm, value: string) =>
    setOwners(owners.map((o, j) => (j === i ? { ...o, [key]: value } : o)));

  async function submit(e: FormEvent) {
    e.preventDefault();
    setError("");
    const amount = Number(form.requested_amount.replace(/[$,\s]/g, ""));
    if (!Number.isInteger(amount) || amount <= 0) {
      setError("Requested amount must be a whole dollar amount, like 250000");
      return;
    }
    setBusy(true);
    try {
      const created = await api<{ id: string }>(
        "POST",
        "/applications",
        {
          ...form,
          requested_amount: amount,
          owners: owners.map((o) => ({ ...o, ownership_percent: Number(o.ownership_percent) })),
        },
        { "Idempotency-Key": idempotencyKey },
      );
      window.location.hash = `#/applications/${created.id}`;
    } catch (err) {
      setError((err as Error).message);
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit}>
      <a href="#/" className="back">
        <BackIcon /> All applications
      </a>
      <div className="page-head">
        <div>
          <h1>New application</h1>
          <p className="sub">Borrower details and owners now; documents are added on the next screen.</p>
        </div>
      </div>

      <Section title="Borrower and request">
        <div className="card-body grid">
          <label>
            Legal name
            <input value={form.legal_name} onChange={set("legal_name")} required maxLength={200} />
          </label>
          <label>
            EIN
            <input value={form.ein} onChange={set("ein")} placeholder="12-3456789" required />
          </label>
          <label className="wide">
            Address
            <input value={form.address} onChange={set("address")} required maxLength={300} />
          </label>
          <label>
            Industry
            <input value={form.industry} onChange={set("industry")} required maxLength={100} />
          </label>
          <label>
            Requested amount (USD)
            <input
              value={form.requested_amount}
              onChange={set("requested_amount")}
              inputMode="numeric"
              placeholder="250000"
              required
            />
          </label>
          <label className="wide">
            Purpose
            <textarea value={form.purpose} onChange={set("purpose")} rows={3} required maxLength={2000} />
          </label>
        </div>
      </Section>

      <Section
        title="Owners"
        foot="SSNs and dates of birth are stored encrypted; lists show only the last four digits."
      >
        <div className="card-body">
          {owners.map((o, i) => (
            <div className="owner-row" key={i}>
              <label>
                Name
                <input value={o.full_name} onChange={(e) => setOwner(i, "full_name", e.target.value)} required />
              </label>
              <label>
                SSN
                <input
                  value={o.ssn}
                  onChange={(e) => setOwner(i, "ssn", e.target.value)}
                  placeholder="123-45-6789"
                  autoComplete="off"
                  required
                />
              </label>
              <label>
                Date of birth
                <input
                  type="date"
                  value={o.date_of_birth}
                  onChange={(e) => setOwner(i, "date_of_birth", e.target.value)}
                  required
                />
              </label>
              <label>
                Ownership %
                <input
                  type="number"
                  min="0.01"
                  max="100"
                  step="0.01"
                  value={o.ownership_percent}
                  onChange={(e) => setOwner(i, "ownership_percent", e.target.value)}
                  required
                />
              </label>
              {owners.length > 1 && (
                <button type="button" onClick={() => setOwners(owners.filter((_, j) => j !== i))}>
                  Remove
                </button>
              )}
            </div>
          ))}
          <button type="button" onClick={() => setOwners([...owners, { ...emptyOwner }])}>
            Add another owner
          </button>
        </div>
      </Section>

      {error && <ErrorMessage>{error}</ErrorMessage>}
      <div className="form-foot">
        <a className="button" href="#/">
          Cancel
        </a>
        <button className="primary" disabled={busy} aria-busy={busy}>
          {busy ? "Saving…" : "Create application"}
        </button>
      </div>
    </form>
  );
}
