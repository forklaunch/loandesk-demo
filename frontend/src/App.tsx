import { FormEvent, useEffect, useState } from "react";
import { api, loadClientSettings, ROLE_LABELS, User } from "./api";
import ApplicationList from "./pages/ApplicationList";
import NewApplication from "./pages/NewApplication";
import ApplicationView from "./pages/ApplicationView";
import Users from "./pages/Users";
import Security from "./pages/Security";
import { ErrorMessage, InfoDot, Mark } from "./ui";

// Tiny hash router: #/, #/new, #/applications/<id>, #/users, #/security
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
  const [totp, setTotp] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      const code = totp.trim();
      onLogin(await api<User>("POST", "/auth/login", { email, password, ...(code ? { totp: code } : {}) }));
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="login">
      <div className="login-card">
        <div className="login-brand">
          <Mark size={42} />
          <div>
            <h1>LoanDesk</h1>
            <div className="muted">Commercial lending workspace</div>
          </div>
        </div>
        <p className="login-lede">
          Loan applications, supporting documents and credit decisions for the commercial lending team.
        </p>
        <form onSubmit={submit}>
          <label>
            Email
            <input
              type="email"
              autoComplete="username"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              autoFocus
              required
            />
          </label>
          <label>
            Password
            <input
              type="password"
              autoComplete="current-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </label>
          <label>
            <span>
              Authenticator code <span className="hint">(only if two-factor sign-in is on)</span>
            </span>
            <input
              inputMode="numeric"
              autoComplete="one-time-code"
              maxLength={6}
              value={totp}
              onChange={(e) => setTotp(e.target.value)}
            />
          </label>
          {error && <ErrorMessage>{error}</ErrorMessage>}
          <button className="primary" disabled={busy} aria-busy={busy}>
            {busy ? "Signing in…" : "Sign in"}
          </button>
        </form>
        <p className="login-foot">Authorised staff only. Sign-ins are recorded in the audit trail.</p>
      </div>
    </div>
  );
}

/** Shown on every screen of a hosted demo, signed in or not. */
function DemoBanner() {
  return (
    <div className="demo-banner" role="note">
      <InfoDot />
      <span>Demo. All data is fake. This copy resets periodically.</span>
    </div>
  );
}

export default function App() {
  const [user, setUser] = useState<User | null | undefined>(undefined);
  const [hostedDemo, setHostedDemo] = useState(false);
  const route = useRoute();

  useEffect(() => {
    // The deployment's settings first (banner, upload route and limit), then the session.
    loadClientSettings()
      .then((settings) => setHostedDemo(settings.hosted_demo))
      .then(() => api<User>("GET", "/auth/me"))
      .then(setUser, () => setUser(null));
    const signedOut = () => setUser(null);
    window.addEventListener("loandesk:signed-out", signedOut);
    return () => window.removeEventListener("loandesk:signed-out", signedOut);
  }, []);

  if (user === undefined) return null;
  if (user === null) {
    return (
      <>
        {hostedDemo && <DemoBanner />}
        <Login
          onLogin={(u) => {
            window.location.hash = "#/";
            setUser(u);
          }}
        />
      </>
    );
  }

  async function logout() {
    // Deliberately ignored: if the request fails (offline, or the session has
    // already ended) the user still leaves the signed-in screens, and the
    // session expires on the server on its own.
    await api("POST", "/auth/logout").catch(() => undefined);
    setUser(null);
  }

  const isAdmin = user.role === "admin";
  let page;
  let narrow = false;
  if (route[0] === "security") {
    page = <Security user={user} onChange={setUser} />;
    narrow = true;
  } else if (isAdmin) page = <Users me={user} />;
  else if (route[0] === "new" && user.role === "relationship_manager") {
    page = <NewApplication />;
    narrow = true;
  } else if (route[0] === "applications" && route[1]) page = <ApplicationView key={route[1]} id={route[1]} />;
  else page = <ApplicationList user={user} />;
  const onSecurity = route[0] === "security";

  return (
    <>
      {hostedDemo && <DemoBanner />}
      <header className="appbar">
        <div className="appbar-inner">
          <a className="brand" href="#/" aria-label="LoanDesk home">
            <Mark />
            <span>LoanDesk</span>
          </a>
          <nav className="appnav" aria-label="Primary">
            {isAdmin ? (
              <a href="#/users" aria-current={onSecurity ? undefined : "page"}>
                Users
              </a>
            ) : (
              <a href="#/" aria-current={onSecurity ? undefined : "page"}>
                Applications
              </a>
            )}
            <a href="#/security" aria-current={onSecurity ? "page" : undefined}>
              Security
            </a>
          </nav>
          <div className="who">
            <div className="who-id">
              <span className="who-name">{user.full_name}</span>
              <span className="who-role">{ROLE_LABELS[user.role]}</span>
            </div>
            <button className="ghost" onClick={logout}>
              Sign out
            </button>
          </div>
        </div>
      </header>
      <main className={narrow ? "narrow" : undefined}>{page}</main>
    </>
  );
}
