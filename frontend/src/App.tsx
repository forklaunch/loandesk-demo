import { FormEvent, useEffect, useState } from "react";
import { api, ROLE_LABELS, User } from "./api";
import ApplicationList from "./pages/ApplicationList";
import NewApplication from "./pages/NewApplication";
import ApplicationView from "./pages/ApplicationView";
import Users from "./pages/Users";

// Tiny hash router: #/, #/new, #/applications/<id>, #/users
function useRoute() {
  const [hash, setHash] = useState(window.location.hash);
  useEffect(() => {
    const onChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return hash.replace(/^#\/?/, "").split("/").filter(Boolean);
}

function Login({ onLogin }: { onLogin: (u: User) => void }) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      onLogin(await api<User>("POST", "/auth/login", { email, password }));
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="login">
      <form className="card" onSubmit={submit}>
        <h1>LoanDesk</h1>
        <p className="muted">Commercial lending workspace</p>
        <label>
          Email
          <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoFocus required />
        </label>
        <label>
          Password
          <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} required />
        </label>
        {error && <div className="error">{error}</div>}
        <button className="primary" disabled={busy}>
          {busy ? "Signing in…" : "Sign in"}
        </button>
      </form>
    </div>
  );
}

export default function App() {
  const [user, setUser] = useState<User | null | undefined>(undefined);
  const route = useRoute();

  useEffect(() => {
    api<User>("GET", "/auth/me").then(setUser, () => setUser(null));
    const signedOut = () => setUser(null);
    window.addEventListener("loandesk:signed-out", signedOut);
    return () => window.removeEventListener("loandesk:signed-out", signedOut);
  }, []);

  if (user === undefined) return null;
  if (user === null) {
    return (
      <Login
        onLogin={(u) => {
          window.location.hash = "#/";
          setUser(u);
        }}
      />
    );
  }

  async function logout() {
    await api("POST", "/auth/logout").catch(() => undefined);
    setUser(null);
  }

  const isAdmin = user.role === "admin";
  let page;
  if (isAdmin) page = <Users me={user} />;
  else if (route[0] === "new" && user.role === "relationship_manager") page = <NewApplication />;
  else if (route[0] === "applications" && route[1]) page = <ApplicationView key={route[1]} id={route[1]} />;
  else page = <ApplicationList user={user} />;

  return (
    <>
      <header className="topbar">
        <a className="brand" href="#/">
          LoanDesk
        </a>
        <nav>{isAdmin ? <a href="#/users">Users</a> : <a href="#/">Applications</a>}</nav>
        <div className="who">
          <span>
            {user.full_name} <span className="muted">· {ROLE_LABELS[user.role]}</span>
          </span>
          <button onClick={logout}>Sign out</button>
        </div>
      </header>
      <main>{page}</main>
    </>
  );
}
