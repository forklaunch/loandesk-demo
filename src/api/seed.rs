//! Demo data. Runs once, only when SEED_DEMO_DATA=true and the users table is
//! empty; refused when APP_ENV=production. Everything here is fictional: SSNs
//! use the 9xx range the SSA never issues, and EINs use prefixes the IRS does
//! not assign. The demo password is supplied by DEMO_PASSWORD, never by code.
//!
//! Seeded rows take the same path as real ones: personal data is sealed,
//! documents are written to object storage as real PDFs and queued, so the
//! worker extracts their figures exactly as it would for an upload.

use crate::{
    domain::{crypto, services::MIN_PASSWORD_LEN},
    error::{AppError, Result, storage},
    registrations::AppState,
};
use forklaunch_core::config;
use shared::{pdfgen, storage::document_key};
use uuid::Uuid;

struct Owner(&'static str, &'static str, (i32, u32, u32), f64);

enum Doc {
    Financial(i64, i64, i64),
    Tax(i64, i64, i64),
    Bank(i64, i64),
}

struct App {
    rm: usize,
    legal_name: &'static str,
    ein: &'static str,
    address: &'static str,
    industry: &'static str,
    amount: i64,
    purpose: &'static str,
    owners: Vec<Owner>,
    docs: Vec<Doc>,
    /// (credit officer index, status, note)
    decision: Option<(usize, &'static str, &'static str)>,
    days_ago: i32,
}

pub async fn run(state: &AppState) -> Result<()> {
    let db = &state.db;
    let members = &db.members;
    let mut tx = db.tx().await?;
    // Serialises two API instances starting at once on an empty database.
    sqlx::query("SELECT pg_advisory_xact_lock(4107001)")
        .execute(&mut *tx)
        .await
        .map_err(storage("lock for seeding"))?;
    let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&mut *tx)
        .await
        .map_err(storage("count users"))?;
    if existing > 0 {
        return Ok(());
    }
    let password: String = config::required("DEMO_PASSWORD")?;
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::InvalidConfig(format!(
            "DEMO_PASSWORD must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    let hash = crypto::hash_password(&password).map_err(storage("hash demo password"))?;

    let users = [
        ("admin@loandesk.demo", "Alex Morgan", "admin"),
        ("maya.chen@loandesk.demo", "Maya Chen", "relationship_manager"),
        ("daniel.ortiz@loandesk.demo", "Daniel Ortiz", "relationship_manager"),
        ("priya.raman@loandesk.demo", "Priya Raman", "credit_officer"),
        ("tom.becker@loandesk.demo", "Tom Becker", "credit_officer"),
    ];
    let mut ids = Vec::new();
    for (email, name, role) in users {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, full_name, role, password_hash) VALUES ($1, $2, $3, $4, $5)")
            .bind(id)
            .bind(email)
            .bind(name)
            .bind(role)
            .bind(&hash)
            .execute(&mut *tx)
            .await
            .map_err(storage("seed user"))?;
        ids.push(id);
    }
    let rms = [ids[1], ids[2]];
    let officers = [ids[3], ids[4]];

    let apps = vec![
        App {
            rm: 0,
            legal_name: "Harborview Brewing Company LLC",
            ein: "07-4410382",
            address: "418 Wharf Street, Portland, ME 04101",
            industry: "Food & Beverage Manufacturing",
            amount: 750_000,
            purpose: "Purchase of a second canning line and expansion of the taproom.",
            owners: vec![
                Owner("Eleanor Whitfield", "900114321", (1978, 4, 12), 60.0),
                Owner("Marcus Whitfield", "900228765", (1981, 9, 3), 40.0),
            ],
            docs: vec![Doc::Financial(4_200_000, 620_000, 900_000), Doc::Bank(312_400, 355_900)],
            decision: None,
            days_ago: 3,
        },
        App {
            rm: 0,
            legal_name: "Ortega Family Dental PLLC",
            ein: "08-2957164",
            address: "2290 Mesa Verde Blvd, Suite 110, Albuquerque, NM 87110",
            industry: "Healthcare - Dental Practice",
            amount: 500_000,
            purpose: "Build-out of a second practice location and purchase of imaging equipment.",
            owners: vec![Owner("Sofia Ortega", "901335502", (1984, 1, 27), 100.0)],
            docs: vec![Doc::Tax(2_100_000, 300_000, 800_000)],
            decision: None,
            days_ago: 6,
        },
        App {
            rm: 1,
            legal_name: "Ironclad Logistics Inc",
            ein: "09-6183047",
            address: "7701 Industrial Parkway, Memphis, TN 38118",
            industry: "Trucking & Freight",
            amount: 600_000,
            purpose: "Refinance existing equipment notes and add six tractors to the fleet.",
            owners: vec![
                Owner("Raymond Kowalski", "902447790", (1969, 11, 8), 51.0),
                Owner("Denise Kowalski", "902447815", (1972, 6, 19), 29.0),
                Owner("Trevor Banks", "903561248", (1988, 2, 14), 20.0),
            ],
            docs: vec![Doc::Financial(1_500_000, 90_000, 1_200_000)],
            decision: Some((
                0,
                "declined",
                "Cash flow does not cover existing debt service, let alone the new notes. \
                 Would reconsider with a smaller request and a guarantor.",
            )),
            days_ago: 21,
        },
        App {
            rm: 1,
            legal_name: "Blue Heron Landscape Supply LLC",
            ein: "17-3048859",
            address: "1560 Chinden Blvd, Boise, ID 83714",
            industry: "Wholesale - Nursery & Garden",
            amount: 250_000,
            purpose: "Seasonal working-capital line for spring inventory.",
            owners: vec![Owner("Hannah Lindqvist", "904672933", (1975, 7, 30), 100.0)],
            docs: vec![Doc::Financial(3_100_000, 410_000, 350_000), Doc::Bank(96_300, 141_750)],
            decision: Some((
                1,
                "approved",
                "Strong coverage and low leverage. Approved as a 12-month revolving line, \
                 annual clean-up required.",
            )),
            days_ago: 14,
        },
        App {
            rm: 0,
            legal_name: "Cedar Ridge Veterinary Clinic PC",
            ein: "18-5521906",
            address: "83 Sweeten Creek Road, Asheville, NC 28803",
            industry: "Veterinary Services",
            amount: 1_200_000,
            purpose: "Acquire the building the clinic currently leases.",
            owners: vec![
                Owner("Naomi Feldman", "905783016", (1980, 3, 5), 70.0),
                Owner("Grant Okafor", "906894127", (1986, 12, 22), 30.0),
            ],
            // No documents: upload the files in sample-docs/ to try the pipeline.
            docs: vec![],
            decision: None,
            days_ago: 1,
        },
        App {
            rm: 1,
            legal_name: "Northgate Precision Machining LLC",
            ein: "19-7730415",
            address: "4025 Kishwaukee Street, Rockford, IL 61109",
            industry: "Manufacturing - Machined Parts",
            amount: 900_000,
            purpose: "Two 5-axis CNC machining centers for a new aerospace contract.",
            owners: vec![
                Owner("Walter Brandt", "907915238", (1964, 5, 17), 50.0),
                Owner("Lucia Brandt-Reyes", "908126349", (1991, 10, 9), 50.0),
            ],
            docs: vec![Doc::Tax(5_600_000, 240_000, 2_400_000)],
            decision: None,
            days_ago: 9,
        },
        App {
            rm: 0,
            legal_name: "Lantern Street Books & Cafe LLC",
            ein: "28-1196273",
            address: "212 Lantern Street, Ann Arbor, MI 48104",
            industry: "Retail - Books & Cafe",
            amount: 150_000,
            purpose: "Storefront renovation and espresso equipment.",
            owners: vec![Owner("Isabel Navarro", "909237450", (1989, 8, 11), 100.0)],
            docs: vec![Doc::Bank(22_150, 31_480)],
            decision: None,
            days_ago: 2,
        },
    ];

    for app in apps {
        let id = Uuid::new_v4();
        let rm = rms[app.rm];
        let tokens = db.index_tokens(
            std::iter::once(app.purpose).chain(app.owners.iter().map(|owner| owner.0)),
        )?;
        sqlx::query(
            "INSERT INTO applications (id, reference, legal_name, ein, address, industry,
                                       requested_amount, purpose_ciphertext, search_tokens,
                                       created_by, created_at)
             VALUES ($1, 'LD-' || nextval('application_ref_seq'), $2, $3, $4, $5, $6, $7, $8, $9,
                     now() - make_interval(days => $10))",
        )
        .bind(id)
        .bind(app.legal_name)
        .bind(app.ein)
        .bind(app.address)
        .bind(app.industry)
        .bind(app.amount)
        .bind(db.seal(&members.purpose, id, app.purpose)?.ciphertext())
        .bind(&tokens)
        .bind(rm)
        .bind(app.days_ago)
        .execute(&mut *tx)
        .await
        .map_err(storage("seed application"))?;

        for Owner(name, ssn, (y, m, d), percent) in &app.owners {
            let owner_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO owners (id, application_id, full_name_ciphertext, ssn_ciphertext,
                                     date_of_birth_ciphertext, ownership_percent)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(owner_id)
            .bind(id)
            .bind(db.seal(&members.owner_name, owner_id, name)?.ciphertext())
            .bind(db.seal(&members.owner_ssn, owner_id, ssn)?.ciphertext())
            .bind(
                db.seal(&members.owner_dob, owner_id, &format!("{y:04}-{m:02}-{d:02}"))?
                    .ciphertext(),
            )
            .bind(percent)
            .execute(&mut *tx)
            .await
            .map_err(storage("seed owner"))?;
        }

        for doc in &app.docs {
            let (doc_type, filename, bytes) = match *doc {
                Doc::Financial(rev, ni, debt) => (
                    "financial_statement",
                    "financial-statement.pdf",
                    pdfgen::financial_statement(app.legal_name, rev, ni, debt),
                ),
                Doc::Tax(rev, ni, debt) => (
                    "tax_return",
                    "business-tax-return.pdf",
                    pdfgen::tax_return(app.legal_name, app.ein, rev, ni, debt),
                ),
                Doc::Bank(open, close) => (
                    "bank_statement",
                    "bank-statement.pdf",
                    pdfgen::bank_statement(app.legal_name, open, close),
                ),
            };
            let doc_id = Uuid::new_v4();
            let key = document_key(db.tenant(), id, doc_id);
            let size = i64::try_from(bytes.len()).unwrap_or(i64::MAX);
            state.store.put(&key, bytes).await?;
            sqlx::query(
                "INSERT INTO documents (id, application_id, doc_type, filename_ciphertext, size_bytes,
                                        storage_key, uploaded_by, uploaded_at, status, request_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, now() - make_interval(days => $8), 'pending', 'seed')",
            )
            .bind(doc_id)
            .bind(id)
            .bind(doc_type)
            .bind(db.seal(&members.filename, doc_id, filename)?.ciphertext())
            .bind(size)
            .bind(&key)
            .bind(rm)
            .bind(app.days_ago)
            .execute(&mut *tx)
            .await
            .map_err(storage("seed document"))?;
            state.queue.enqueue(&mut tx, doc_id, id).await?;
        }

        if let Some((officer, status, note)) = app.decision {
            sqlx::query(
                "UPDATE applications SET status = $1, decided_by = $2, decision_note_ciphertext = $3,
                        decided_at = created_at + interval '2 days'
                 WHERE id = $4",
            )
            .bind(status)
            .bind(officers[officer])
            .bind(db.seal(&members.decision_note, id, note)?.ciphertext())
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(storage("seed decision"))?;
        }
    }
    db.audit(&mut tx, None, "demo_data_seeded", None, None).await?;
    tx.commit().await.map_err(storage("seed demo data"))?;
    tracing::info!(event = "demo_data_seeded", "seeded demo users and applications");
    Ok(())
}
