CREATE TABLE users (
    id             UUID PRIMARY KEY,
    email          TEXT NOT NULL UNIQUE,
    full_name      TEXT NOT NULL,
    role           TEXT NOT NULL CHECK (role IN ('relationship_manager', 'credit_officer', 'admin')),
    password_hash  TEXT NOT NULL,
    failed_logins  INT NOT NULL DEFAULT 0,
    locked_until   TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Only a SHA-256 of the session token is stored.
CREATE TABLE sessions (
    token_hash  TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE SEQUENCE application_ref_seq START 1001;

-- Money columns are whole US dollars.
CREATE TABLE applications (
    id                UUID PRIMARY KEY,
    reference         TEXT NOT NULL UNIQUE,
    legal_name        TEXT NOT NULL,
    ein               TEXT NOT NULL,
    address           TEXT NOT NULL,
    industry          TEXT NOT NULL,
    requested_amount  BIGINT NOT NULL CHECK (requested_amount > 0),
    purpose           TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'submitted' CHECK (status IN ('submitted', 'approved', 'declined')),
    -- Filled in by the worker from uploaded documents.
    revenue           BIGINT,
    net_income        BIGINT,
    total_debt        BIGINT,
    created_by        UUID NOT NULL REFERENCES users (id),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_by        UUID REFERENCES users (id),
    decided_at        TIMESTAMPTZ,
    decision_note     TEXT,
    -- Nobody decides an application they created, whatever their role is now.
    CONSTRAINT no_self_decision CHECK (decided_by IS NULL OR decided_by <> created_by)
);
CREATE INDEX applications_created_by_idx ON applications (created_by);

CREATE TABLE owners (
    id                 UUID PRIMARY KEY,
    application_id     UUID NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    full_name          TEXT NOT NULL,
    -- AES-256-GCM, nonce || ciphertext. Only the last four digits are kept in clear.
    ssn_encrypted      BYTEA NOT NULL,
    ssn_last4          TEXT NOT NULL,
    date_of_birth      DATE NOT NULL,
    ownership_percent  DOUBLE PRECISION NOT NULL CHECK (ownership_percent > 0 AND ownership_percent <= 100)
);
CREATE INDEX owners_application_idx ON owners (application_id);

-- A document row is also its own processing job (status / attempts / locked_at).
CREATE TABLE documents (
    id              UUID PRIMARY KEY,
    application_id  UUID NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    doc_type        TEXT NOT NULL CHECK (doc_type IN ('tax_return', 'bank_statement', 'financial_statement')),
    filename        TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    storage_key     TEXT NOT NULL UNIQUE,
    uploaded_by     UUID NOT NULL REFERENCES users (id),
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'done', 'failed')),
    attempts        INT NOT NULL DEFAULT 0,
    locked_at       TIMESTAMPTZ,
    processed_at    TIMESTAMPTZ,
    error           TEXT,
    revenue         BIGINT,
    net_income      BIGINT,
    total_debt      BIGINT
);
CREATE INDEX documents_application_idx ON documents (application_id);
CREATE INDEX documents_queue_idx ON documents (uploaded_at) WHERE status IN ('pending', 'processing');

CREATE TABLE audit_log (
    id         BIGSERIAL PRIMARY KEY,
    at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_id    UUID,
    action     TEXT NOT NULL,
    entity_id  UUID,
    detail     TEXT
);
