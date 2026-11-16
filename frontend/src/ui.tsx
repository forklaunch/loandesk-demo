import { ReactNode } from "react";

/**
 * Small shared pieces of the interface. Icons are inline SVG: the page's
 * Content-Security-Policy allows no images from elsewhere and no data: URIs.
 * Every icon is decorative (`aria-hidden`); the words next to it carry the meaning.
 */
function Icon({ size = 16, children }: { size?: number; children: ReactNode }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}

/** The product mark: three rising columns of a ledger. */
export function Mark({ size = 22 }: { size?: number }) {
  return (
    <svg
      className="mark"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
      focusable="false"
    >
      <rect x="3" y="13" width="4" height="8" rx="1" fill="currentColor" opacity="0.55" />
      <rect x="10" y="8" width="4" height="13" rx="1" fill="currentColor" opacity="0.8" />
      <rect x="17" y="3" width="4" height="18" rx="1" fill="currentColor" />
    </svg>
  );
}

export const SearchIcon = () => (
  <Icon>
    <circle cx="7" cy="7" r="4.5" />
    <path d="M10.5 10.5 14 14" />
  </Icon>
);
export const BackIcon = () => (
  <Icon size={14}>
    <path d="M9.5 3.5 5 8l4.5 4.5" />
  </Icon>
);
export const UploadIcon = () => (
  <Icon size={18}>
    <path d="M8 10.5V2.5M4.75 5.5 8 2.25l3.25 3.25M2.5 10.5v2a1 1 0 0 0 1 1h9a1 1 0 0 0 1-1v-2" />
  </Icon>
);
export const AlertIcon = () => (
  <Icon>
    <circle cx="8" cy="8" r="6.25" />
    <path d="M8 4.75v3.75M8 11v.25" />
  </Icon>
);
export const CheckIcon = () => (
  <Icon>
    <circle cx="8" cy="8" r="6.25" />
    <path d="m5.25 8.25 1.9 1.9 3.6-4" />
  </Icon>
);
export const InfoDot = () => (
  <Icon size={14}>
    <circle cx="8" cy="8" r="6.25" />
    <path d="M8 7.25v4M8 4.75V5" />
  </Icon>
);

/** An error from the server or the form, announced to screen readers. */
export function ErrorMessage({ children }: { children: ReactNode }) {
  return (
    <div className="error" role="alert">
      <AlertIcon />
      <span>{children}</span>
    </div>
  );
}

/** Something that worked. */
export function Notice({ children }: { children: ReactNode }) {
  return (
    <div className="notice" role="status">
      <CheckIcon />
      <span>{children}</span>
    </div>
  );
}

/** A titled block of a page. */
export function Section({
  title,
  aside,
  children,
  foot,
}: {
  title: ReactNode;
  aside?: ReactNode;
  children: ReactNode;
  foot?: ReactNode;
}) {
  return (
    <section className="card">
      <div className="card-head">
        <h2>{title}</h2>
        {aside}
      </div>
      {children}
      {foot && <div className="card-foot muted small">{foot}</div>}
    </section>
  );
}
