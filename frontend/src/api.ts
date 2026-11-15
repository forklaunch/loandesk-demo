export type Role = "relationship_manager" | "credit_officer" | "admin";

export interface User {
  id: string;
  email: string;
  full_name: string;
  role: Role;
  created_at?: string;
  mfa_enabled?: boolean;
}

export interface Risk {
  dscr: number | null;
  leverage: number | null;
  annual_debt_service: number | null;
  flag: "low" | "medium" | "high" | null;
  missing: string[];
}

export interface Application {
  id: string;
  reference: string;
  legal_name: string;
  ein: string;
  address: string;
  industry: string;
  requested_amount: number;
  /** Stored encrypted; sent with a single application, not in lists. */
  purpose: string | null;
  status: "submitted" | "approved" | "declined";
  revenue: number | null;
  net_income: number | null;
  total_debt: number | null;
  created_by_name: string;
  created_at: string;
  decided_by_name: string | null;
  decided_at: string | null;
  decision_note: string | null;
  risk: Risk;
}

export interface Owner {
  id: string;
  full_name: string;
  ssn_last4: string;
  date_of_birth: string;
  ownership_percent: number;
}

export interface Doc {
  id: string;
  doc_type: string;
  filename: string;
  size_bytes: number;
  status: "pending" | "processing" | "done" | "failed";
  error: string | null;
  revenue: number | null;
  net_income: number | null;
  total_debt: number | null;
  uploaded_at: string;
  uploaded_by_name: string;
}

export interface ApplicationDetail {
  application: Application;
  owners: Owner[];
  documents: Doc[];
  permissions: { can_upload: boolean; can_decide: boolean; is_creator: boolean };
}

export const ROLE_LABELS: Record<Role, string> = {
  relationship_manager: "Relationship manager",
  credit_officer: "Credit officer",
  admin: "Admin",
};

export const DOC_TYPE_LABELS: Record<string, string> = {
  tax_return: "Tax return",
  bank_statement: "Bank statement",
  financial_statement: "Financial statement",
};

/** The API answers within 30 seconds or gives up; the browser waits a little longer. */
const REQUEST_TIMEOUT_MS = 35_000;
/** An upload grant is valid for five minutes. */
const UPLOAD_TIMEOUT_MS = 300_000;

/** Every call the app makes has a deadline, so a stalled connection ends in a message, not a spinner. */
async function withTimeout(ms: number, call: (signal: AbortSignal) => Promise<Response>): Promise<Response> {
  try {
    return await call(AbortSignal.timeout(ms));
  } catch (error) {
    if (error instanceof DOMException && (error.name === "TimeoutError" || error.name === "AbortError")) {
      throw new Error("The server took too long to answer. Check your connection and try again.");
    }
    throw error;
  }
}

async function request(
  method: string,
  path: string,
  body?: unknown,
  extraHeaders?: Record<string, string>,
): Promise<{ data: unknown; headers: Headers }> {
  // The API rejects state-changing requests without this header (CSRF guard).
  const headers: Record<string, string> = { "X-Requested-With": "loandesk", ...extraHeaders };
  let payload: BodyInit | undefined;
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
    payload = JSON.stringify(body);
  }
  const res = await withTimeout(REQUEST_TIMEOUT_MS, (signal) =>
    fetch("/api" + path, { method, headers, body: payload, credentials: "same-origin", signal }),
  );
  const isJson = res.headers.get("content-type")?.includes("application/json");
  const data = isJson ? await res.json() : null;
  if (!res.ok) {
    if (res.status === 401 && !path.startsWith("/auth/")) {
      window.dispatchEvent(new Event("loandesk:signed-out"));
    }
    const fallback = res.status === 413 ? TOO_LARGE : `Request failed (${res.status})`;
    throw new Error(data?.error ?? fallback);
  }
  return { data, headers: res.headers };
}

export async function api<T>(
  method: string,
  path: string,
  body?: unknown,
  headers?: Record<string, string>,
): Promise<T> {
  return (await request(method, path, body, headers)).data as T;
}

/** One page of a list. `next` is the cursor for the following page, if any. */
export interface Page<T> {
  items: T[];
  next: string | null;
}

export async function apiPage<T>(method: string, path: string, body?: unknown): Promise<Page<T>> {
  const res = await request(method, path, body);
  return { items: res.data as T[], next: res.headers.get("x-next-cursor") };
}

export const MAX_UPLOAD_BYTES = 20 * 1024 * 1024;
const TOO_LARGE = "The file is too large (limit is 20 MB)";

interface UploadGrant {
  id: string;
  url: string;
  fields: Record<string, string>;
}

/**
 * Uploads one PDF. The file goes straight to document storage with a
 * short-lived grant from the API (it never passes through the API server);
 * the API then checks what arrived and queues it for reading.
 */
export async function uploadDocument(applicationId: string, docType: string, file: File): Promise<void> {
  if (file.size > MAX_UPLOAD_BYTES) throw new Error(TOO_LARGE);
  const grant = await api<UploadGrant>("POST", `/applications/${applicationId}/documents`, {
    doc_type: docType,
    filename: file.name,
    size_bytes: file.size,
  });
  const form = new FormData();
  for (const [name, value] of Object.entries(grant.fields)) form.append(name, value);
  // Storage requires the declared type; the API verifies the real content afterwards.
  form.append("file", new Blob([file], { type: "application/pdf" }), "document.pdf");
  const stored = await withTimeout(UPLOAD_TIMEOUT_MS, (signal) =>
    fetch(grant.url, { method: "POST", body: form, credentials: "omit", signal }),
  );
  if (!stored.ok) {
    throw new Error(stored.status === 400 || stored.status === 413 ? TOO_LARGE : "The upload was interrupted. Try again.");
  }
  await api("POST", `/documents/${grant.id}/complete`);
}

/** Opens a stored document through a one-minute link issued (and audited) by the API. */
export async function downloadDocument(documentId: string): Promise<void> {
  const { url } = await api<{ url: string }>("POST", `/documents/${documentId}/download-url`);
  window.location.assign(url);
}

export const money = (v: number | null | undefined) =>
  v === null || v === undefined
    ? "—"
    : v.toLocaleString("en-US", { style: "currency", currency: "USD", maximumFractionDigits: 0 });

export const date = (iso: string | null) => (iso ? new Date(iso).toLocaleDateString() : "—");
