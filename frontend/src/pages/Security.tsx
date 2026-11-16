import { FormEvent, useState } from "react";
import { api, User } from "../api";
import { ErrorMessage, Notice, Section } from "../ui";

/** Two-factor sign-in: enrol an authenticator app, then confirm with a code. */
export default function Security({ user, onChange }: { user: User; onChange: (u: User) => void }) {
  const [secret, setSecret] = useState<{ secret: string; otpauth_url: string } | null>(null);
  const [code, setCode] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [passwords, setPasswords] = useState({ current_password: "", new_password: "", repeat: "" });
  const [passwordError, setPasswordError] = useState("");
  const [passwordChanged, setPasswordChanged] = useState(false);

  async function changePassword(e: FormEvent) {
    e.preventDefault();
    setPasswordError("");
    setPasswordChanged(false);
    if (passwords.new_password !== passwords.repeat) {
      setPasswordError("The two new passwords do not match");
      return;
    }
    setBusy(true);
    try {
      await api("POST", "/auth/password", {
        current_password: passwords.current_password,
        new_password: passwords.new_password,
      });
      setPasswords({ current_password: "", new_password: "", repeat: "" });
      setPasswordChanged(true);
    } catch (err) {
      setPasswordError((err as Error).message);
    } finally {
      setBusy(false);
    }
  }

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
        <div>
          <h1>Security</h1>
          <p className="sub">Two-factor sign-in and your password.</p>
        </div>
      </div>
      <Section
        title="Two-factor sign-in"
        aside={
          user.mfa_enabled ? <span className="badge status-approved">On</span> : <span className="badge">Off</span>
        }
      >
        <div className="card-body">
          {error && <ErrorMessage>{error}</ErrorMessage>}
          {user.mfa_enabled ? (
            <p>You are asked for a code from your authenticator app each time you sign in.</p>
          ) : secret ? (
            <form className="stack" onSubmit={confirm}>
              <p>
                Add this key to an authenticator app (or open <a href={secret.otpauth_url}>this link</a> on the device
                that has one), then enter the 6-digit code it shows.
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
              <div className="actions">
                <button className="primary" disabled={busy} aria-busy={busy}>
                  {busy ? "Checking…" : "Turn on"}
                </button>
              </div>
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
        </div>
      </Section>
      <form onSubmit={changePassword}>
        <Section title="Password">
          <div className="card-body">
            {passwordError && <ErrorMessage>{passwordError}</ErrorMessage>}
            {passwordChanged && <Notice>Password changed. Every other session of yours has been signed out.</Notice>}
            <div className="grid three">
              <label>
                Current password
                <input
                  type="password"
                  autoComplete="current-password"
                  value={passwords.current_password}
                  onChange={(e) => setPasswords({ ...passwords, current_password: e.target.value })}
                  required
                />
              </label>
              <label>
                New password (12+ characters)
                <input
                  type="password"
                  autoComplete="new-password"
                  minLength={12}
                  value={passwords.new_password}
                  onChange={(e) => setPasswords({ ...passwords, new_password: e.target.value })}
                  required
                />
              </label>
              <label>
                New password again
                <input
                  type="password"
                  autoComplete="new-password"
                  minLength={12}
                  value={passwords.repeat}
                  onChange={(e) => setPasswords({ ...passwords, repeat: e.target.value })}
                  required
                />
              </label>
            </div>
            <div className="form-foot">
              <button className="primary" disabled={busy} aria-busy={busy}>
                Change password
              </button>
            </div>
          </div>
        </Section>
      </form>
    </>
  );
}
