/**
 * Reports errors that happen in a user's browser, so a broken screen is
 * noticed by the team and not only by the person looking at it.
 *
 * Reports go to the LoanDesk API (`POST /api/client-errors`), which writes one
 * structured `browser_error` log line per report into the same pipeline as the
 * services' own errors, where an alert rule watches it.
 *
 * What is sent: the kind of failure, the error class ("TypeError"), the screen
 * with record ids removed, the stack frames (function, script, line) and the
 * build. What is never sent: the error message (messages can quote what
 * someone typed), form contents, or the address bar. The API reduces every
 * field again before logging it.
 */

declare const __APP_VERSION__: string;

type Kind = "error" | "unhandledrejection" | "render";

/** At most this many reports per page load: a render loop must not flood the API. */
const MAX_REPORTS = 5;
const REPORT_TIMEOUT_MS = 5_000;
let sent = 0;
const seen = new Set<string>();

/** "#/applications/7f3c…" -> "#/applications/:id". */
export function screenName(hash: string): string {
  const parts = hash
    .replace(/^#\/?/, "")
    .split(/[?&]/)[0]
    .split("/")
    .filter(Boolean)
    .map((part) => (/^[a-z-]{1,24}$/.test(part) ? part : ":id"));
  return "#/" + parts.join("/");
}

/** Keeps stack frames and drops the first line, which repeats the message. */
export function stackFrames(stack: string | undefined): string {
  return (stack ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("at ") || line.includes("@"))
    .slice(0, 20)
    .join("\n")
    .slice(0, 4000);
}

export function reportError(kind: Kind, error: unknown, componentStack?: string | null): void {
  try {
    const known = error instanceof Error ? error : undefined;
    const stack = stackFrames(componentStack ?? known?.stack);
    const report = {
      kind,
      class: (known?.name ?? "Error").slice(0, 80),
      screen: screenName(window.location.hash).slice(0, 120),
      stack,
      release: __APP_VERSION__,
    };
    const signature = `${report.kind}|${report.class}|${report.screen}|${stack.slice(0, 200)}`;
    if (sent >= MAX_REPORTS || seen.has(signature)) return;
    seen.add(signature);
    sent += 1;
    void fetch("/api/client-errors", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Requested-With": "loandesk" },
      body: JSON.stringify(report),
      credentials: "same-origin",
      keepalive: true,
      signal: AbortSignal.timeout(REPORT_TIMEOUT_MS),
    }).catch(() => {
      // Reporting is best effort: it must never cause a second error.
    });
  } catch {
    // As above.
  }
}

/** Called once, from the entry point, before the app renders. */
export function installErrorReporting(): void {
  window.addEventListener("error", (event) => reportError("error", event.error));
  window.addEventListener("unhandledrejection", (event) => reportError("unhandledrejection", event.reason));
}
