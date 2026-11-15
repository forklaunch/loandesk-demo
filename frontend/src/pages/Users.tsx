import { FormEvent, useEffect, useState } from "react";
import { api, date, Role, ROLE_LABELS, User } from "../api";

const ROLES = Object.keys(ROLE_LABELS) as Role[];
const emptyForm = { full_name: "", email: "", password: "", role: "relationship_manager" as Role };

export default function Users({ me }: { me: User }) {
  const [users, setUsers] = useState<User[]>([]);
  const [form, setForm] = useState(emptyForm);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const load = () => api<User[]>("GET", "/users").then(setUsers, (e: Error) => setError(e.message));
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

  return (
    <>
      <div className="page-head">
        <h1>Users</h1>
      </div>
      {error && <div className="error">{error}</div>}
      <div className="card table-wrap">
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Email</th>
              <th>Role</th>
              <th>Added</th>
            </tr>
          </thead>
          <tbody>
            {users.map((u) => (
              <tr key={u.id} className="static">
                <td>{u.full_name}</td>
                <td>{u.email}</td>
                <td>
                  {u.id === me.id ? (
                    <span title="You cannot change your own role">{ROLE_LABELS[u.role]} (you)</span>
                  ) : (
                    <select value={u.role} onChange={(e) => setRole(u.id, e.target.value as Role)} aria-label={`Role for ${u.full_name}`}>
                      {ROLES.map((r) => (
                        <option key={r} value={r}>
                          {ROLE_LABELS[r]}
                        </option>
                      ))}
                    </select>
                  )}
                </td>
                <td>{date(u.created_at ?? null)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <form className="card" onSubmit={add}>
        <h2>Add a user</h2>
        <div className="grid">
          <label>
            Full name
            <input value={form.full_name} onChange={(e) => setForm({ ...form, full_name: e.target.value })} required />
          </label>
          <label>
            Email
            <input type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} required />
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
        <button className="primary" disabled={busy}>
          Add user
        </button>
      </form>
    </>
  );
}
