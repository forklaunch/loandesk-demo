import { FormEvent, useState } from "react";
import { api, User } from "../api";

/** Two-factor sign-in: enrol an authenticator app, then confirm with a code. */
export default function Security({ user, onChange }: { user: User; onChange: (u: User) => void }) {
  const [secret, setSecret] = useState<{ secret: string; otpauth_url: string } | null>(null);
  const [code, setCode] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function start() {
    setError("");
    try {
      setSecret(await api("POST", "/auth/mfa/enrol"));
    } catch (err) {
      setError((err as Error).message);
    }
  }

  async function confirm(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      await api("POST", "/auth/mfa/confirm", { code: code.trim() });
      setSecret(null);
      setCode("");
      onChange({ ...user, mfa_enabled: true });
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <div className="page-head">
        <h1>Security</h1>
      </div>
      <section className="card">
        <h2>Two-factor sign-in</h2>
        {error && <div className="error">{error}</div>}
        {user.mfa_enabled ? (
          <p>
            <span className="badge status-approved">On</span> You are asked for a code from your authenticator app
            each time you sign in.
          </p>
        ) : secret ? (
          <form onSubmit={confirm}>
            <p>
              Add this key to an authenticator app (or open{" "}
              <a href={secret.otpauth_url}>this link</a> on the device that has one), then enter the 6-digit code it
              shows.
            </p>
            <p>
              <code>{secret.secret}</code>
            </p>
            <label>
              Code
              <input
                inputMode="numeric"
                autoComplete="one-time-code"
                maxLength={6}
                value={code}
                onChange={(e) => setCode(e.target.value)}
                required
              />
            </label>
            <button className="primary" disabled={busy}>
              {busy ? "Checking…" : "Turn on"}
            </button>
          </form>
        ) : (
          <>
            <p className="muted">
              Off. With two-factor sign-in, a password alone is not enough to get into your account.
            </p>
            <button className="primary" onClick={start}>
              Set up
            </button>
          </>
        )}
      </section>
    </>
  );
}
