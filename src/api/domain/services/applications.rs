//! Loan applications: who may see them, creating, deciding, and the audited
//! reads of personal data.
use crate::{
    api::{
        policy::{CREDIT_OFFICER, CurrentUser, RELATIONSHIP_MANAGER},
        transport,
    },
    domain::schemas::{
        applications::{
            Application, ApplicationDetail, ApplicationExport, Created, Decided, Decision,
            NewApplication, Owner, Permissions, RevealedSsn, Risk,
        },
        documents::Document,
    },
    error::{AppError, Result, bad, storage},
    persistence::{Db, Disclosures, Tx, entities::StoredOwner},
};
use chrono::{DateTime, NaiveDate, SubsecRound, Utc};
use forklaunch_core::persistence::protected::storage::ProtectedModel;
use shared::risk::{self, RiskFlag};
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

const DEFAULT_PAGE: i64 = 50;
const MAX_PAGE: i64 = 100;
/// An application is created with at most this many owners.
const MAX_OWNERS: usize = 20;
/// Upper bound on the owners read for one application, whatever the table holds.
const OWNERS_READ_LIMIT: i64 = 100;

/// The application columns every query selects, with a literal tail. A macro
/// (not string formatting) so each statement stays a compile-time constant:
/// values only ever reach SQL as bind parameters.
macro_rules! select_application {
    ($tail:literal) => {
        concat!(
            "SELECT a.id, a.reference, a.legal_name, a.ein, a.address, a.industry, a.requested_amount,
                    a.purpose_ciphertext, a.status, a.revenue, a.net_income, a.total_debt, a.created_by,
                    rm.full_name AS created_by_name, a.created_at,
                    co.full_name AS decided_by_name, a.decided_at, a.decision_note_ciphertext
             FROM applications a
             JOIN users rm ON rm.id = a.created_by
             LEFT JOIN users co ON co.id = a.decided_by ",
            $tail
        )
    };
}

#[derive(Clone)]
pub struct ApplicationsService {
    db: Db,
}

/// An application row with its two encrypted texts still sealed.
pub struct ApplicationRow {
    pub id: Uuid,
    /// Full-precision creation time, for the keyset cursor.
    created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub status: String,
    purpose_ciphertext: Option<String>,
    decision_note_ciphertext: Option<String>,
    summary: Application,
}

fn risk_summary(
    requested: i64,
    revenue: Option<i64>,
    net_income: Option<i64>,
    total_debt: Option<i64>,
) -> Risk {
    let summary = risk::assess(requested, revenue, net_income, total_debt);
    Risk {
        dscr: summary.dscr,
        leverage: summary.leverage,
        annual_debt_service: summary.annual_debt_service,
        flag: summary.flag.map(|flag| {
            match flag {
                RiskFlag::Low => "low",
                RiskFlag::Medium => "medium",
                RiskFlag::High => "high",
            }
            .to_owned()
        }),
        missing: summary.missing.iter().map(|name| (*name).to_owned()).collect(),
    }
}

fn application_row(row: &PgRow) -> ApplicationRow {
    let id: Uuid = row.get("id");
    let created_by: Uuid = row.get("created_by");
    let status: String = row.get("status");
    let requested_amount: i64 = row.get("requested_amount");
    let (revenue, net_income, total_debt) =
        (row.get("revenue"), row.get("net_income"), row.get("total_debt"));
    let created_at: DateTime<Utc> = row.get("created_at");
    ApplicationRow {
        id,
        created_at,
        created_by,
        status: status.clone(),
        purpose_ciphertext: row.get("purpose_ciphertext"),
        decision_note_ciphertext: row.get("decision_note_ciphertext"),
        summary: Application {
            id: id.to_string(),
            reference: row.get("reference"),
            legal_name: row.get("legal_name"),
            ein: row.get("ein"),
            address: row.get("address"),
            industry: row.get("industry"),
            requested_amount,
            purpose: None,
            status,
            revenue,
            net_income,
            total_debt,
            created_by: created_by.to_string(),
            created_by_name: row.get("created_by_name"),
            created_at: wire_time(created_at),
            decided_by_name: row.get("decided_by_name"),
            decided_at: row.get::<Option<DateTime<Utc>>, _>("decided_at").map(wire_time),
            decision_note: None,
            risk: risk_summary(requested_amount, revenue, net_income, total_debt),
        },
    }
}

/// The API contract carries timestamps to the millisecond.
pub fn wire_time(at: DateTime<Utc>) -> DateTime<Utc> {
    at.trunc_subsecs(3)
}

/// Who may see applications at all:
/// - relationship managers: only the ones they created
/// - credit officers: all
/// - admins: none (they manage users, they do not need borrower data)
fn visibility_filter(user: &CurrentUser) -> Result<Option<Uuid>> {
    if user.is(RELATIONSHIP_MANAGER) {
        Ok(Some(user.id))
    } else if user.is(CREDIT_OFFICER) {
        Ok(None)
    } else {
        Err(AppError::Forbidden(
            "Admins manage users and do not have access to applications".into(),
        ))
    }
}

fn like_pattern(q: &str) -> String {
    let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{escaped}%")
}

fn text(label: &str, value: &str, max: usize) -> Result<String> {
    let v = value.trim();
    if v.is_empty() {
        return Err(bad(format!("{label} is required")));
    }
    if v.chars().count() > max {
        return Err(bad(format!("{label} must be at most {max} characters")));
    }
    Ok(v.to_string())
}

/// Returns just the digits if `value` is exactly `count` digits once dashes
/// and spaces are removed.
fn digits(value: &str, count: usize) -> Option<String> {
    let d: String = value.chars().filter(|c| !matches!(c, '-' | ' ')).collect();
    (d.len() == count && d.chars().all(|c| c.is_ascii_digit())).then_some(d)
}

/// Opaque keyset cursor for (created_at DESC, id DESC).
fn encode_cursor(created_at: DateTime<Utc>, id: &str) -> String {
    format!("{}.{id}", created_at.timestamp_micros())
}
fn decode_cursor(cursor: &str) -> Result<(DateTime<Utc>, Uuid)> {
    let invalid = || bad("That page link is no longer valid");
    let (micros, id) = cursor.split_once('.').ok_or_else(invalid)?;
    let at = DateTime::from_timestamp_micros(micros.parse().map_err(|_| invalid())?)
        .ok_or_else(invalid)?;
    Ok((at, Uuid::parse_str(id).map_err(|_| invalid())?))
}

impl ApplicationsService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Loads an application the user is allowed to see. Anything else is a 404,
    /// so ids of other people's applications cannot be probed.
    pub async fn load_visible(
        &self,
        tx: &mut Tx<'_>,
        user: &CurrentUser,
        id: Uuid,
    ) -> Result<ApplicationRow> {
        let only_creator = visibility_filter(user)?;
        let row = sqlx::query(select_application!(
            "WHERE a.id = $1 AND ($2::uuid IS NULL OR a.created_by = $2)"
        ))
        .bind(id)
        .bind(only_creator)
        .fetch_optional(&mut **tx)
        .await
        .map_err(storage("load application"))?;
        match row {
            Some(row) => Ok(application_row(&row)),
            None => {
                tracing::warn!(
                    event = "permission_denied",
                    user_id = %user.id,
                    role = %user.role,
                    action = "read application",
                    resource = %id,
                    reason = "not found or not visible to this user",
                    request_id = %transport::request_id(),
                    "permission denied"
                );
                Err(AppError::NotFound)
            }
        }
    }

    /// One page, newest first, ordered by the unique key (created_at, id).
    /// The same query serves the plain list (`q` = None) and search.
    pub async fn list(
        &self,
        user: &CurrentUser,
        q: Option<String>,
        limit: Option<i64>,
        cursor: Option<String>,
    ) -> Result<(Vec<Application>, Option<String>)> {
        let only_creator = visibility_filter(user)?;
        let q = q.as_deref().map(str::trim).filter(|q| !q.is_empty());
        if q.is_some_and(|q| q.chars().count() > 100) {
            return Err(bad("Search text is too long"));
        }
        let pattern = q.map(like_pattern);
        // Lets "074410382" find EIN "07-4410382".
        let ein_pattern = q
            .map(|q| q.replace('-', ""))
            .filter(|q| !q.is_empty() && q.chars().all(|c| c.is_ascii_digit()))
            .map(|q| format!("%{q}%"));
        // Purpose and owner names are encrypted; they are matched through keyed word tokens.
        // While a key rotation is in progress, rows not yet re-indexed still
        // carry tokens under the previous key: either set matches.
        let (tokens, previous_tokens) = match q {
            Some(q) => self.db.query_tokens(q)?.map(|t| (t.active, t.previous)).unzip(),
            None => (None, None),
        };
        let previous_tokens = previous_tokens.flatten();
        let limit = limit.unwrap_or(DEFAULT_PAGE).clamp(1, MAX_PAGE);
        let (after_at, after_id) = cursor.map(|cursor| decode_cursor(&cursor)).transpose()?.unzip();

        let mut tx = self.db.tx().await?;
        let rows = sqlx::query(select_application!(
            r"WHERE ($1::uuid IS NULL OR a.created_by = $1)
              AND ($2::text IS NULL
                   OR a.legal_name ILIKE $2 ESCAPE '\'
                   OR a.reference ILIKE $2 ESCAPE '\'
                   OR a.ein ILIKE $2 ESCAPE '\'
                   OR replace(a.ein, '-', '') LIKE $3
                   OR a.address ILIKE $2 ESCAPE '\'
                   OR a.industry ILIKE $2 ESCAPE '\'
                   OR a.status ILIKE $2 ESCAPE '\'
                   OR rm.full_name ILIKE $2 ESCAPE '\'
                   OR ($4::text[] IS NOT NULL AND a.search_tokens @> $4)
                   OR ($8::text[] IS NOT NULL AND a.search_tokens @> $8))
              AND ($5::timestamptz IS NULL OR (a.created_at, a.id) < ($5, $6))
            ORDER BY a.created_at DESC, a.id DESC
            LIMIT $7"
        ))
        .bind(only_creator)
        .bind(pattern)
        .bind(ein_pattern)
        .bind(tokens)
        .bind(after_at)
        .bind(after_id)
        .bind(limit + 1)
        .bind(previous_tokens)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("list applications"))?;
        tx.commit().await.map_err(storage("list applications"))?;

        let mut page: Vec<ApplicationRow> = rows.iter().map(application_row).collect();
        let next = if page.len() > usize::try_from(limit).unwrap_or(usize::MAX) {
            page.pop();
            page.last().map(|last| encode_cursor(last.created_at, &last.id.to_string()))
        } else {
            None
        };
        Ok((page.into_iter().map(|row| row.summary).collect(), next))
    }

    pub async fn create(
        &self,
        user: &CurrentUser,
        body: NewApplication,
        idempotency_key: Option<String>,
    ) -> Result<Created> {
        let legal_name = text("Legal name", &body.legal_name, 200)?;
        let address = text("Address", &body.address, 300)?;
        let industry = text("Industry", &body.industry, 100)?;
        let purpose = text("Purpose", &body.purpose, 2000)?;
        let ein = digits(&body.ein, 9)
            .map(|d| format!("{}-{}", &d[..2], &d[2..]))
            .ok_or_else(|| bad("EIN must be 9 digits, like 12-3456789"))?;
        if body.requested_amount <= 0 || body.requested_amount > 1_000_000_000_000 {
            return Err(bad("Requested amount must be a positive dollar amount"));
        }
        if body.owners.is_empty() || body.owners.len() > MAX_OWNERS {
            return Err(bad("Add between 1 and 20 owners"));
        }
        let today = Utc::now().date_naive();
        let mut owners = Vec::new();
        let mut total_percent = 0.0;
        for o in &body.owners {
            let name = text("Owner name", &o.full_name, 200)?;
            // Never echo the submitted SSN back in an error message.
            let ssn =
                digits(&o.ssn, 9).ok_or_else(|| bad(format!("SSN for {name} must be 9 digits")))?;
            let dob = NaiveDate::parse_from_str(o.date_of_birth.trim(), "%Y-%m-%d")
                .ok()
                .filter(|dob| today.years_since(*dob).is_some_and(|age| (18..=120).contains(&age)))
                .ok_or_else(|| {
                    bad(format!(
                        "Date of birth for {name} is not valid (owners must be 18 or older)"
                    ))
                })?;
            if !(o.ownership_percent > 0.0 && o.ownership_percent <= 100.0) {
                return Err(bad(format!("Ownership percent for {name} must be between 0 and 100")));
            }
            total_percent += o.ownership_percent;
            owners.push((name, ssn, dob, o.ownership_percent));
        }
        if total_percent > 100.001 {
            return Err(bad("Ownership percentages add up to more than 100"));
        }
        let idempotency_key =
            idempotency_key.map(|key| key.trim().to_owned()).filter(|key| !key.is_empty());

        let id = Uuid::new_v4();
        let members = &self.db.members;
        let sealed_purpose = self.db.seal(&members.purpose, id, &purpose)?;
        let tokens = self.db.index_tokens(
            std::iter::once(purpose.as_str()).chain(owners.iter().map(|o| o.0.as_str())),
        )?;

        let mut tx = self.db.tx().await?;
        // A retry carrying the same Idempotency-Key gets the first result back;
        // the unique index makes that hold even when two retries race.
        let reference: Option<String> = sqlx::query_scalar(
            "INSERT INTO applications (id, reference, legal_name, ein, address, industry,
                                       requested_amount, purpose_ciphertext, search_tokens,
                                       idempotency_key, created_by)
             VALUES ($1, 'LD-' || nextval('application_ref_seq'), $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (tenant_id, created_by, idempotency_key) WHERE idempotency_key IS NOT NULL
             DO NOTHING
             RETURNING reference",
        )
        .bind(id)
        .bind(&legal_name)
        .bind(&ein)
        .bind(&address)
        .bind(&industry)
        .bind(body.requested_amount)
        .bind(sealed_purpose.ciphertext())
        .bind(&tokens)
        .bind(&idempotency_key)
        .bind(user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("create application"))?;
        let Some(reference) = reference else {
            let existing = sqlx::query(
                "SELECT id, reference FROM applications WHERE created_by = $1 AND idempotency_key = $2",
            )
            .bind(user.id)
            .bind(&idempotency_key)
            .fetch_one(&mut *tx)
            .await
            .map_err(storage("load idempotent result"))?;
            tx.commit().await.map_err(storage("load idempotent result"))?;
            return Ok(Created {
                id: existing.get::<Uuid, _>("id").to_string(),
                reference: existing.get("reference"),
            });
        };

        for (name, ssn, dob, percent) in owners {
            let owner_id = Uuid::new_v4();
            let row = StoredOwner {
                id: owner_id,
                tenant_id: self.db.tenant().to_string(),
                application_id: id,
                full_name_ciphertext: self.db.seal(&members.owner_name, owner_id, &name)?,
                ssn_ciphertext: self.db.seal(&members.owner_ssn, owner_id, &ssn)?,
                date_of_birth_ciphertext: self.db.seal(
                    &members.owner_dob,
                    owner_id,
                    &dob.to_string(),
                )?,
                ownership_percent: percent,
            };
            row.validate_protected_storage().map_err(storage("validate owner storage"))?;
            sqlx::query(
                "INSERT INTO owners (id, application_id, full_name_ciphertext, ssn_ciphertext,
                                     date_of_birth_ciphertext, ownership_percent)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(row.id)
            .bind(row.application_id)
            .bind(row.full_name_ciphertext.ciphertext())
            .bind(row.ssn_ciphertext.ciphertext())
            .bind(row.date_of_birth_ciphertext.ciphertext())
            .bind(row.ownership_percent)
            .execute(&mut *tx)
            .await
            .map_err(storage("create owner"))?;
        }
        self.db.audit(&mut tx, Some(user.id), "application_created", Some(id), None).await?;
        tx.commit().await.map_err(storage("create application"))?;
        tracing::info!(
            event = "application_created",
            application = %id,
            user_id = %user.id,
            request_id = %transport::request_id(),
            "application created"
        );
        Ok(Created { id: id.to_string(), reference })
    }

    async fn reveal_application(
        &self,
        row: ApplicationRow,
        user: &CurrentUser,
        reason: &'static str,
        disclosures: &Disclosures,
    ) -> Result<Application> {
        let members = &self.db.members;
        let mut application = row.summary;
        if let Some(ciphertext) = row.purpose_ciphertext {
            application.purpose = Some(
                self.db
                    .reveal(&members.purpose, row.id, ciphertext, user.id, reason, disclosures)
                    .await?,
            );
        }
        if let Some(ciphertext) = row.decision_note_ciphertext {
            application.decision_note = Some(
                self.db
                    .reveal(
                        &members.decision_note,
                        row.id,
                        ciphertext,
                        user.id,
                        reason,
                        disclosures,
                    )
                    .await?,
            );
        }
        Ok(application)
    }

    async fn owners(
        &self,
        tx: &mut Tx<'_>,
        application: Uuid,
        user: &CurrentUser,
        reason: &'static str,
        disclosures: &Disclosures,
    ) -> Result<Vec<Owner>> {
        let members = &self.db.members;
        let rows = sqlx::query(
            "SELECT id, full_name_ciphertext, ssn_ciphertext, date_of_birth_ciphertext, ownership_percent
             FROM owners WHERE application_id = $1 ORDER BY ownership_percent DESC, id LIMIT $2",
        )
        .bind(application)
        .bind(OWNERS_READ_LIMIT)
        .fetch_all(&mut **tx)
        .await
        .map_err(storage("load owners"))?;
        let mut owners = Vec::with_capacity(rows.len());
        for row in rows {
            let id: Uuid = row.get("id");
            owners.push(Owner {
                id: id.to_string(),
                full_name: self
                    .db
                    .reveal(
                        &members.owner_name,
                        id,
                        row.get("full_name_ciphertext"),
                        user.id,
                        reason,
                        disclosures,
                    )
                    .await?,
                // Last four only, taken from the sealed value's disclosure projection.
                ssn_last4: self.db.ssn_last4(id, row.get("ssn_ciphertext"))?,
                date_of_birth: self
                    .db
                    .reveal(
                        &members.owner_dob,
                        id,
                        row.get("date_of_birth_ciphertext"),
                        user.id,
                        reason,
                        disclosures,
                    )
                    .await?,
                ownership_percent: row.get("ownership_percent"),
            });
        }
        owners.sort_by(|a, b| {
            b.ownership_percent
                .total_cmp(&a.ownership_percent)
                .then_with(|| a.full_name.cmp(&b.full_name))
        });
        Ok(owners)
    }

    async fn documents(
        &self,
        tx: &mut Tx<'_>,
        application: Uuid,
        user: &CurrentUser,
        reason: &'static str,
        disclosures: &Disclosures,
    ) -> Result<Vec<Document>> {
        let rows = sqlx::query(
            "SELECT d.id, d.doc_type, d.filename_ciphertext, d.size_bytes, d.status, d.error, d.revenue,
                    d.net_income, d.total_debt, d.uploaded_at, u.full_name AS uploaded_by_name
             FROM documents d JOIN users u ON u.id = d.uploaded_by
             WHERE d.application_id = $1 AND d.status <> 'uploading'
             ORDER BY d.uploaded_at, d.id
             LIMIT 200",
        )
        .bind(application)
        .fetch_all(&mut **tx)
        .await
        .map_err(storage("load documents"))?;
        let mut documents = Vec::with_capacity(rows.len());
        for row in rows {
            let id: Uuid = row.get("id");
            documents.push(Document {
                id: id.to_string(),
                doc_type: row.get("doc_type"),
                filename: self
                    .db
                    .reveal(
                        &self.db.members.filename,
                        id,
                        row.get("filename_ciphertext"),
                        user.id,
                        reason,
                        disclosures,
                    )
                    .await?,
                size_bytes: row.get("size_bytes"),
                status: row.get("status"),
                error: row.get("error"),
                revenue: row.get("revenue"),
                net_income: row.get("net_income"),
                total_debt: row.get("total_debt"),
                uploaded_at: wire_time(row.get("uploaded_at")),
                uploaded_by_name: row.get("uploaded_by_name"),
            });
        }
        Ok(documents)
    }

    pub async fn detail(&self, user: &CurrentUser, id: &str) -> Result<ApplicationDetail> {
        const REASON: &str = "view-application";
        let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        let row = self.load_visible(&mut tx, user, id).await?;
        let open = row.status == "submitted";
        let is_creator = row.created_by == user.id;
        let disclosures = Disclosures::new();
        let owners = self.owners(&mut tx, id, user, REASON, &disclosures).await?;
        let documents = self.documents(&mut tx, id, user, REASON, &disclosures).await?;
        let application = self.reveal_application(row, user, REASON, &disclosures).await?;
        // Reading a borrower's record is itself recorded, field by field.
        disclosures.persist(&mut tx, user.id).await?;
        tx.commit().await.map_err(storage("record application view"))?;
        Ok(ApplicationDetail {
            application,
            owners,
            documents,
            permissions: Permissions {
                can_upload: open && is_creator,
                can_decide: open && !is_creator && user.is(CREDIT_OFFICER),
                is_creator,
            },
        })
    }

    pub async fn decide(&self, user: &CurrentUser, id: &str, body: Decision) -> Result<Decided> {
        let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
        if !matches!(body.decision.as_str(), "approved" | "declined") {
            return Err(bad("Decision must be 'approved' or 'declined'"));
        }
        let note = text("Note", &body.note, 2000)?;
        let mut tx = self.db.tx().await?;
        let app = self.load_visible(&mut tx, user, id).await?;
        if app.created_by == user.id {
            tracing::warn!(
                event = "permission_denied",
                user_id = %user.id,
                role = %user.role,
                action = "decide application",
                resource = %id,
                reason = "creator cannot decide their own application",
                request_id = %transport::request_id(),
                "permission denied"
            );
            return Err(AppError::Forbidden(
                "You cannot approve or decline an application you created".into(),
            ));
        }
        let sealed_note = self.db.seal(&self.db.members.decision_note, id, &note)?;
        // The WHERE clause repeats both rules so two simultaneous decisions, or a
        // decision racing a role change, cannot slip through.
        let updated = sqlx::query(
            "UPDATE applications
             SET status = $1, decided_by = $2, decided_at = now(), decision_note_ciphertext = $3
             WHERE id = $4 AND status = 'submitted' AND created_by <> $2",
        )
        .bind(&body.decision)
        .bind(user.id)
        .bind(sealed_note.ciphertext())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(storage("decide application"))?
        .rows_affected();
        if updated == 0 {
            return Err(AppError::Conflict("This application has already been decided".into()));
        }
        self.db
            .audit(&mut tx, Some(user.id), "application_decided", Some(id), Some(&body.decision))
            .await?;
        tx.commit().await.map_err(storage("decide application"))?;
        // One of two fixed words, never the request text.
        let outcome = if body.decision == "approved" { "approved" } else { "declined" };
        tracing::info!(
            event = "application_decided",
            application = %id,
            decision = outcome,
            user_id = %user.id,
            request_id = %transport::request_id(),
            "application decided"
        );
        Ok(Decided { status: body.decision })
    }

    /// Returns one owner's full SSN. The audit entry is committed before the
    /// value leaves this function; if it cannot be written, nothing is returned.
    pub async fn reveal_ssn(
        &self,
        user: &CurrentUser,
        id: &str,
        owner_id: &str,
    ) -> Result<RevealedSsn> {
        let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
        let owner_id = Uuid::parse_str(owner_id).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        self.load_visible(&mut tx, user, id).await?;
        let ciphertext: String = sqlx::query_scalar(
            "SELECT ssn_ciphertext FROM owners WHERE id = $1 AND application_id = $2",
        )
        .bind(owner_id)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("load owner"))?
        .ok_or(AppError::NotFound)?;
        let disclosures = Disclosures::new();
        let ssn = self
            .db
            .reveal(
                &self.db.members.owner_ssn,
                owner_id,
                ciphertext,
                user.id,
                "reveal-ssn",
                &disclosures,
            )
            .await?;
        self.db.audit(&mut tx, Some(user.id), "ssn_revealed", Some(owner_id), None).await?;
        disclosures.persist(&mut tx, user.id).await?;
        tx.commit().await.map_err(storage("record ssn reveal"))?;
        // Also a named log event, so an unusual number of reveals can alert.
        tracing::info!(
            event = "ssn_revealed",
            user_id = %user.id,
            application = %id,
            owner = %owner_id,
            request_id = %transport::request_id(),
            "full SSN revealed to an authorised user"
        );
        if ssn.len() != 9 {
            return Err(AppError::Storage("stored ssn has an unexpected shape".into()));
        }
        Ok(RevealedSsn { ssn: format!("{}-{}-{}", &ssn[..3], &ssn[3..5], &ssn[5..]) })
    }

    /// Everything held about one application, for a data-subject or portability
    /// request. Limited to people who may already see it, and audited.
    pub async fn export(&self, user: &CurrentUser, id: &str) -> Result<ApplicationExport> {
        const REASON: &str = "export-application";
        let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
        let mut tx = self.db.tx().await?;
        let row = self.load_visible(&mut tx, user, id).await?;
        let disclosures = Disclosures::new();
        let owners = self.owners(&mut tx, id, user, REASON, &disclosures).await?;
        let documents = self.documents(&mut tx, id, user, REASON, &disclosures).await?;
        let application = self.reveal_application(row, user, REASON, &disclosures).await?;
        self.db.audit(&mut tx, Some(user.id), "application_exported", Some(id), None).await?;
        disclosures.persist(&mut tx, user.id).await?;
        tx.commit().await.map_err(storage("record export"))?;
        tracing::info!(
            event = "application_exported",
            user_id = %user.id,
            application = %id,
            request_id = %transport::request_id(),
            "application exported"
        );
        Ok(ApplicationExport { exported_at: wire_time(Utc::now()), application, owners, documents })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trips_to_the_microsecond() {
        let at = DateTime::from_timestamp_micros(1_763_000_000_123_456).unwrap();
        let id = Uuid::new_v4();
        assert_eq!(decode_cursor(&encode_cursor(at, &id.to_string())).unwrap(), (at, id));
        assert!(decode_cursor("x.y").is_err());
    }

    #[test]
    fn digits_accepts_only_the_exact_count() {
        assert_eq!(digits("12-3456789", 9).as_deref(), Some("123456789"));
        assert_eq!(digits("900 11 4321", 9).as_deref(), Some("900114321"));
        assert!(digits("12345678", 9).is_none());
        assert!(digits("12345678a", 9).is_none());
    }

    #[test]
    fn like_patterns_escape_wildcards() {
        assert_eq!(like_pattern("50%_a\\"), "%50\\%\\_a\\\\\\\\%".replace("\\\\\\\\", "\\\\"));
    }
}
