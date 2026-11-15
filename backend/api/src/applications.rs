use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared::risk::{self, RiskSummary};
use uuid::Uuid;

use crate::{
    auth::{audit, CurrentUser, Role},
    documents::DocumentRow,
    error::{bad, ApiResult, AppError},
    AppState,
};

#[derive(Serialize, sqlx::FromRow)]
pub struct ApplicationRow {
    pub id: Uuid,
    pub reference: String,
    pub legal_name: String,
    pub ein: String,
    pub address: String,
    pub industry: String,
    pub requested_amount: i64,
    pub purpose: String,
    pub status: String,
    pub revenue: Option<i64>,
    pub net_income: Option<i64>,
    pub total_debt: Option<i64>,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub created_at: DateTime<Utc>,
    pub decided_by_name: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_note: Option<String>,
}

const SELECT_APPLICATION: &str = "
    SELECT a.id, a.reference, a.legal_name, a.ein, a.address, a.industry, a.requested_amount,
           a.purpose, a.status, a.revenue, a.net_income, a.total_debt, a.created_by,
           rm.full_name AS created_by_name, a.created_at,
           co.full_name AS decided_by_name, a.decided_at, a.decision_note
    FROM applications a
    JOIN users rm ON rm.id = a.created_by
    LEFT JOIN users co ON co.id = a.decided_by";

#[derive(Serialize)]
pub struct WithRisk {
    #[serde(flatten)]
    application: ApplicationRow,
    risk: RiskSummary,
}

impl From<ApplicationRow> for WithRisk {
    fn from(a: ApplicationRow) -> Self {
        let risk = risk::assess(a.requested_amount, a.revenue, a.net_income, a.total_debt);
        WithRisk { application: a, risk }
    }
}

/// Who may see applications at all:
/// - relationship managers: only the ones they created
/// - credit officers: all
/// - admins: none (they manage users, they do not need borrower data)
fn visibility_filter(user: &CurrentUser) -> ApiResult<Option<Uuid>> {
    match user.role {
        Role::RelationshipManager => Ok(Some(user.id)),
        Role::CreditOfficer => Ok(None),
        Role::Admin => Err(AppError::Forbidden(
            "Admins manage users and do not have access to applications",
        )),
    }
}

/// Loads an application the user is allowed to see. Anything else is a 404,
/// so ids of other people's applications cannot be probed.
pub async fn load_visible(st: &AppState, user: &CurrentUser, id: Uuid) -> ApiResult<ApplicationRow> {
    let only_creator = visibility_filter(user)?;
    sqlx::query_as::<_, ApplicationRow>(&format!(
        "{SELECT_APPLICATION} WHERE a.id = $1 AND ($2::uuid IS NULL OR a.created_by = $2)"
    ))
    .bind(id)
    .bind(only_creator)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)
}

fn like_pattern(q: &str) -> String {
    let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{escaped}%")
}

#[derive(Deserialize)]
pub struct ListParams {
    q: Option<String>,
}

pub async fn list(
    State(st): State<AppState>,
    user: CurrentUser,
    Query(params): Query<ListParams>,
) -> ApiResult<Json<Vec<WithRisk>>> {
    let only_creator = visibility_filter(&user)?;
    let q = params.q.as_deref().map(str::trim).filter(|q| !q.is_empty());
    if q.is_some_and(|q| q.len() > 100) {
        return Err(bad("Search text is too long"));
    }
    let pattern = q.map(like_pattern);
    // Lets "074410382" find EIN "07-4410382".
    let ein_pattern = q
        .map(|q| q.replace('-', ""))
        .filter(|q| !q.is_empty() && q.chars().all(|c| c.is_ascii_digit()))
        .map(|q| format!("%{q}%"));

    let rows = sqlx::query_as::<_, ApplicationRow>(&format!(
        r"{SELECT_APPLICATION}
        WHERE ($1::uuid IS NULL OR a.created_by = $1)
          AND ($2::text IS NULL
               OR a.legal_name ILIKE $2 ESCAPE '\'
               OR a.reference ILIKE $2 ESCAPE '\'
               OR a.ein ILIKE $2 ESCAPE '\'
               OR replace(a.ein, '-', '') LIKE $3
               OR a.address ILIKE $2 ESCAPE '\'
               OR a.industry ILIKE $2 ESCAPE '\'
               OR a.purpose ILIKE $2 ESCAPE '\'
               OR a.status ILIKE $2 ESCAPE '\'
               OR rm.full_name ILIKE $2 ESCAPE '\'
               OR EXISTS (SELECT 1 FROM owners o
                          WHERE o.application_id = a.id AND o.full_name ILIKE $2 ESCAPE '\'))
        ORDER BY a.created_at DESC
        LIMIT 200"
    ))
    .bind(only_creator)
    .bind(pattern)
    .bind(ein_pattern)
    .fetch_all(&st.db)
    .await?;

    Ok(Json(rows.into_iter().map(WithRisk::from).collect()))
}

#[derive(Deserialize)]
pub struct NewOwner {
    full_name: String,
    ssn: String,
    date_of_birth: NaiveDate,
    ownership_percent: f64,
}

#[derive(Deserialize)]
pub struct NewApplication {
    legal_name: String,
    ein: String,
    address: String,
    industry: String,
    requested_amount: i64,
    purpose: String,
    owners: Vec<NewOwner>,
}

fn text(label: &str, value: &str, max: usize) -> ApiResult<String> {
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

pub async fn create(
    State(st): State<AppState>,
    user: CurrentUser,
    Json(body): Json<NewApplication>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    user.require_role(
        Role::RelationshipManager,
        "Only relationship managers can create applications",
    )?;

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
    if body.owners.is_empty() || body.owners.len() > 20 {
        return Err(bad("Add between 1 and 20 owners"));
    }

    let today = Utc::now().date_naive();
    let mut owners = Vec::new();
    let mut total_percent = 0.0;
    for o in &body.owners {
        let name = text("Owner name", &o.full_name, 200)?;
        // Never echo the submitted SSN back in an error message.
        let ssn = digits(&o.ssn, 9).ok_or_else(|| bad(format!("SSN for {name} must be 9 digits")))?;
        if today.years_since(o.date_of_birth).is_none_or(|age| !(18..=120).contains(&age)) {
            return Err(bad(format!("Date of birth for {name} is not valid (owners must be 18 or older)")));
        }
        if !(o.ownership_percent > 0.0 && o.ownership_percent <= 100.0) {
            return Err(bad(format!("Ownership percent for {name} must be between 0 and 100")));
        }
        total_percent += o.ownership_percent;
        owners.push((name, ssn, o.date_of_birth, o.ownership_percent));
    }
    if total_percent > 100.001 {
        return Err(bad("Ownership percentages add up to more than 100"));
    }

    let id = Uuid::new_v4();
    let mut tx = st.db.begin().await?;
    let reference: String = sqlx::query_scalar(
        "INSERT INTO applications (id, reference, legal_name, ein, address, industry,
                                   requested_amount, purpose, created_by)
         VALUES ($1, 'LD-' || nextval('application_ref_seq'), $2, $3, $4, $5, $6, $7, $8)
         RETURNING reference",
    )
    .bind(id)
    .bind(&legal_name)
    .bind(&ein)
    .bind(&address)
    .bind(&industry)
    .bind(body.requested_amount)
    .bind(&purpose)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    for (name, ssn, dob, percent) in owners {
        sqlx::query(
            "INSERT INTO owners (id, application_id, full_name, ssn_encrypted, ssn_last4,
                                 date_of_birth, ownership_percent)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(name)
        .bind(st.cipher.encrypt(&ssn)?)
        .bind(&ssn[5..])
        .bind(dob)
        .bind(percent)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    audit(&st.db, Some(user.id), "application_created", Some(id), None).await?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id, "reference": reference }))))
}

#[derive(Serialize, sqlx::FromRow)]
struct OwnerRow {
    id: Uuid,
    full_name: String,
    ssn_last4: String,
    date_of_birth: NaiveDate,
    ownership_percent: f64,
}

pub async fn detail(
    State(st): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let app = load_visible(&st, &user, id).await?;

    let owners: Vec<OwnerRow> = sqlx::query_as(
        "SELECT id, full_name, ssn_last4, date_of_birth, ownership_percent
         FROM owners WHERE application_id = $1 ORDER BY ownership_percent DESC, full_name",
    )
    .bind(id)
    .fetch_all(&st.db)
    .await?;

    let documents: Vec<DocumentRow> = sqlx::query_as(
        "SELECT d.id, d.doc_type, d.filename, d.size_bytes, d.status, d.error, d.revenue,
                d.net_income, d.total_debt, d.uploaded_at, u.full_name AS uploaded_by_name
         FROM documents d JOIN users u ON u.id = d.uploaded_by
         WHERE d.application_id = $1 ORDER BY d.uploaded_at",
    )
    .bind(id)
    .fetch_all(&st.db)
    .await?;

    let open = app.status == "submitted";
    let is_creator = app.created_by == user.id;
    let permissions = json!({
        "can_upload": open && is_creator,
        "can_decide": open && !is_creator && user.role == Role::CreditOfficer,
        "is_creator": is_creator,
    });

    Ok(Json(json!({
        "application": WithRisk::from(app),
        "owners": owners,
        "documents": documents,
        "permissions": permissions,
    })))
}

#[derive(Deserialize)]
pub struct Decision {
    decision: String,
    note: String,
}

pub async fn decide(
    State(st): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<Decision>,
) -> ApiResult<Json<Value>> {
    user.require_role(
        Role::CreditOfficer,
        "Only credit officers can approve or decline applications",
    )?;
    if !matches!(body.decision.as_str(), "approved" | "declined") {
        return Err(bad("Decision must be 'approved' or 'declined'"));
    }
    let note = text("Note", &body.note, 2000)?;
    let app = load_visible(&st, &user, id).await?;
    if app.created_by == user.id {
        return Err(AppError::Forbidden(
            "You cannot approve or decline an application you created",
        ));
    }

    // The WHERE clause repeats both rules so two simultaneous decisions, or a
    // decision racing a role change, cannot slip through.
    let updated = sqlx::query(
        "UPDATE applications
         SET status = $1, decided_by = $2, decided_at = now(), decision_note = $3
         WHERE id = $4 AND status = 'submitted' AND created_by <> $2",
    )
    .bind(&body.decision)
    .bind(user.id)
    .bind(&note)
    .bind(id)
    .execute(&st.db)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::Conflict("This application has already been decided"));
    }
    audit(&st.db, Some(user.id), "application_decided", Some(id), Some(&body.decision)).await?;
    Ok(Json(json!({ "status": body.decision })))
}

/// Returns one owner's full SSN. Every call is written to the audit log
/// before the value is returned.
pub async fn reveal_ssn(
    State(st): State<AppState>,
    user: CurrentUser,
    Path((id, owner_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    load_visible(&st, &user, id).await?;
    let encrypted: Vec<u8> =
        sqlx::query_scalar("SELECT ssn_encrypted FROM owners WHERE id = $1 AND application_id = $2")
            .bind(owner_id)
            .bind(id)
            .fetch_optional(&st.db)
            .await?
            .ok_or(AppError::NotFound)?;
    audit(&st.db, Some(user.id), "ssn_revealed", Some(owner_id), None).await?;
    let ssn = st.cipher.decrypt(&encrypted)?;
    Ok(Json(json!({
        "ssn": format!("{}-{}-{}", &ssn[..3], &ssn[3..5], &ssn[5..])
    })))
}
