//! Demo data. Runs once, only when SEED_DEMO_DATA=true and the users table is
//! empty. Everything here is fictional: SSNs use the 9xx range the SSA never
//! issues, and EINs use prefixes the IRS does not assign.
//!
//! Seeded documents are written as real PDFs and left "pending", so the
//! worker extracts their figures exactly as it would for an upload.

use anyhow::Context;
use chrono::NaiveDate;
use shared::pdfgen;
use uuid::Uuid;

use crate::{crypto, users::MIN_PASSWORD_LEN, AppState};

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

pub async fn run(st: &AppState) -> anyhow::Result<()> {
    let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&st.db)
        .await?;
    if existing > 0 {
        return Ok(());
    }
    let password = std::env::var("DEMO_PASSWORD").context("DEMO_PASSWORD is required when seeding")?;
    anyhow::ensure!(
        password.chars().count() >= MIN_PASSWORD_LEN,
        "DEMO_PASSWORD must be at least {MIN_PASSWORD_LEN} characters"
    );
    let hash = crypto::hash_password(&password)?;

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
            .execute(&st.db)
            .await?;
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
        sqlx::query(
            "INSERT INTO applications (id, reference, legal_name, ein, address, industry,
                                       requested_amount, purpose, created_by, created_at)
             VALUES ($1, 'LD-' || nextval('application_ref_seq'), $2, $3, $4, $5, $6, $7, $8,
                     now() - make_interval(days => $9))",
        )
        .bind(id)
        .bind(app.legal_name)
        .bind(app.ein)
        .bind(app.address)
        .bind(app.industry)
        .bind(app.amount)
        .bind(app.purpose)
        .bind(rm)
        .bind(app.days_ago)
        .execute(&st.db)
        .await?;

        for Owner(name, ssn, (y, m, d), percent) in &app.owners {
            sqlx::query(
                "INSERT INTO owners (id, application_id, full_name, ssn_encrypted, ssn_last4,
                                     date_of_birth, ownership_percent)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(name)
            .bind(st.cipher.encrypt(ssn)?)
            .bind(&ssn[5..])
            .bind(NaiveDate::from_ymd_opt(*y, *m, *d).context("bad seed date")?)
            .bind(percent)
            .execute(&st.db)
            .await?;
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
            let storage_key = format!("{doc_id}.pdf");
            tokio::fs::write(st.docs_dir.join(&storage_key), &bytes).await?;
            sqlx::query(
                "INSERT INTO documents (id, application_id, doc_type, filename, size_bytes,
                                        storage_key, uploaded_by, uploaded_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, now() - make_interval(days => $8))",
            )
            .bind(doc_id)
            .bind(id)
            .bind(doc_type)
            .bind(filename)
            .bind(bytes.len() as i64)
            .bind(&storage_key)
            .bind(rm)
            .bind(app.days_ago)
            .execute(&st.db)
            .await?;
        }

        if let Some((officer, status, note)) = app.decision {
            sqlx::query(
                "UPDATE applications SET status = $1, decided_by = $2, decision_note = $3,
                        decided_at = created_at + interval '2 days'
                 WHERE id = $4",
            )
            .bind(status)
            .bind(officers[officer])
            .bind(note)
            .bind(id)
            .execute(&st.db)
            .await?;
        }
    }

    tracing::info!("seeded demo users and applications");
    Ok(())
}
