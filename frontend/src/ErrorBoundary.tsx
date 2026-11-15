import { Component, ErrorInfo, ReactNode } from "react";

/**
 * Keeps a rendering error from blanking the whole app: shows a fallback and
 * records the error. Only the error name, message and component stack are
 * recorded: never form contents or anything a user typed.
 */
export default class ErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("LoanDesk render error", { name: error.name, message: error.message, stack: info.componentStack });
  }

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <main>
        <div className="card">
          <h1>Something went wrong</h1>
          <p className="muted">This screen could not be shown. Your data has not been changed.</p>
          <button className="primary" onClick={() => window.location.reload()}>
            Reload
          </button>
        </div>
      </main>
    );
  }
}
