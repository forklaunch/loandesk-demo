import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import ErrorBoundary from "./ErrorBoundary";
import { installErrorReporting } from "./errorReporting";
import "./styles.css";

// Uncaught errors and rejected promises are reported from the first moment.
installErrorReporting();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
);
