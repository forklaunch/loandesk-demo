export type Role = "relationship_manager" | "credit_officer" | "admin";

export interface User {
  id: string;
  email: string;
  full_name: string;
  role: Role;
  created_at?: string;
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
  purpose: string;
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

export async function api<T>(method: string, path: string, body?: unknown): Promise<T> {
  // The API rejects state-changing requests without this header (CSRF guard).
  const headers: Record<string, string> = { "X-Requested-With": "loandesk" };
  let payload: BodyInit | undefined;
  if (body instanceof FormData) {
    payload = body;
  } else if (body !== undefined) {
    headers["Content-Type"] = "application/json";
    payload = JSON.stringify(body);
  }
  const res = await fetch("/api" + path, { method, headers, body: payload, credentials: "same-origin" });
  const isJson = res.headers.get("content-type")?.includes("application/json");
  const data = isJson ? await res.json() : null;
  if (!res.ok) {
    if (res.status === 401 && !path.startsWith("/auth/")) {
      window.dispatchEvent(new Event("loandesk:signed-out"));
    }
    const fallback = res.status === 413 ? "The file is too large (limit is 20 MB)" : `Request failed (${res.status})`;
    throw new Error(data?.error ?? fallback);
  }
  return data as T;
}

export const money = (v: number | null | undefined) =>
  v === null || v === undefined
    ? "—"
    : v.toLocaleString("en-US", { style: "currency", currency: "USD", maximumFractionDigits: 0 });

export const date = (iso: string | null) => (iso ? new Date(iso).toLocaleDateString() : "—");
