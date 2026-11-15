mod applications;
mod auth;
mod crypto;
mod documents;
mod error;
mod seed;
mod users;

use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{bail, Context};
use axum::{
    extract::{DefaultBodyLimit, Request},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

/// The key shipped in docker-compose.yml for local demos. Refused in production.
const DEV_ENCRYPTION_KEY: &str = "ZGV2LW9ubHkta2V5LWRvLW5vdC11c2UtaW4tcHJvZCE=";

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cipher: Arc<crypto::Cipher>,
    pub docs_dir: PathBuf,
    pub cookie_secure: bool,
    pub dummy_hash: Arc<String>,
}

fn env(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("missing environment variable {name}"))
}

fn env_flag(name: &str) -> bool {
    matches!(std::env::var(name).as_deref(), Ok("true") | Ok("1"))
}

async fn connect_db() -> anyhow::Result<PgPool> {
    let opts = PgConnectOptions::new()
        .host(&env("DB_HOST")?)
        .port(std::env::var("DB_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5432))
        .username(&env("DB_USER")?)
        .password(&env("DB_PASSWORD")?)
        .database(&env("DB_NAME")?);
    let mut last = None;
    for _ in 0..30 {
        match PgPoolOptions::new().max_connections(10).connect_with(opts.clone()).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    bail!("could not connect to the database: {last:?}")
}

/// Every state-changing request must carry a custom header. Browsers will not
/// send it cross-site without a CORS preflight (which we never allow), so this
/// backs up the SameSite=Strict cookie against CSRF. Also marks every API
/// response as uncacheable since most of them contain borrower data.
async fn guard(req: Request, next: Next) -> Response {
    let safe = matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    let has_header = req
        .headers()
        .get("x-requested-with")
        .is_some_and(|v| v == "loandesk");
    let mut res = if safe || has_header {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Missing X-Requested-With header" })),
        )
            .into_response()
    };
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    res
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // `loandesk-api gen-samples <dir>` writes the sample PDFs and exits.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("gen-samples") {
        let dir = PathBuf::from(args.get(2).context("usage: gen-samples <dir>")?);
        for (name, bytes) in shared::pdfgen::sample_documents() {
            std::fs::write(dir.join(name), bytes)?;
            println!("wrote {name}");
        }
        return Ok(());
    }

    let production = std::env::var("APP_ENV").as_deref() == Ok("production");
    let key = env("DATA_ENCRYPTION_KEY")?;
    let seed_demo = env_flag("SEED_DEMO_DATA");
    let cookie_secure = env_flag("COOKIE_SECURE");
    if production {
        if key.trim() == DEV_ENCRYPTION_KEY {
            bail!("refusing to start in production with the development DATA_ENCRYPTION_KEY");
        }
        if seed_demo {
            bail!("refusing to seed demo logins in production (unset SEED_DEMO_DATA)");
        }
        if !cookie_secure {
            bail!("COOKIE_SECURE=true is required in production (serve over HTTPS)");
        }
    }

    let docs_dir = PathBuf::from(env("DOCS_DIR")?);
    tokio::fs::create_dir_all(&docs_dir).await?;

    let db = connect_db().await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        cipher: Arc::new(crypto::Cipher::from_base64(&key)?),
        docs_dir,
        cookie_secure,
        dummy_hash: Arc::new(crypto::hash_password("not-a-real-password")?),
    };

    if seed_demo {
        seed::run(&state).await.context("seeding demo data")?;
    }

    let app = Router::new()
        .route("/api/healthz", get(|| async { "ok" }))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route(
            "/api/applications",
            get(applications::list).post(applications::create),
        )
        .route("/api/applications/{id}", get(applications::detail))
        .route("/api/applications/{id}/decision", post(applications::decide))
        .route(
            "/api/applications/{id}/owners/{owner_id}/reveal-ssn",
            post(applications::reveal_ssn),
        )
        .route(
            "/api/applications/{id}/documents",
            post(documents::upload)
                .layer(DefaultBodyLimit::max(documents::MAX_UPLOAD_BYTES + 64 * 1024)),
        )
        .route("/api/documents/{id}/download", get(documents::download))
        .route("/api/users", get(users::list).post(users::create))
        .route("/api/users/{id}", patch(users::set_role))
        .layer(middleware::from_fn(guard))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("LoanDesk API listening on :8080");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
