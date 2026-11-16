//! Checks on the repository itself: build files, CI, monitoring and source
//! conventions. They need no services. Each test is named after the
//! requirement it demonstrates; `docs/controls.md` maps requirements to tests.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("repository root")
}
fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|_| panic!("{path} is readable"))
}
fn exists(path: &str) -> bool {
    root().join(path).exists()
}

/// Hand-written source files under `dir` with one of `extensions`.
fn files(dir: &str, extensions: &[&str]) -> Vec<(String, String)> {
    fn walk(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("{} is readable", dir.display())) {
            let path = entry.expect("directory entry").path();
            let name = path.file_name().expect("file name").to_string_lossy().into_owned();
            if path.is_dir() {
                if !["target", "node_modules", "dist", ".git"].contains(&name.as_str()) {
                    walk(&path, extensions, out);
                }
            } else if extensions.iter().any(|extension| name.ends_with(extension)) {
                out.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&root().join(dir), extensions, &mut found);
    found.sort();
    found
        .into_iter()
        .map(|path| {
            let name =
                path.strip_prefix(root()).expect("in repository").to_string_lossy().into_owned();
            (name, fs::read_to_string(&path).unwrap_or_default())
        })
        .collect()
}

/// The application's own code: the API, the worker, the shared crate, the
/// front end and the scripts. (`src/core`, `src/monitoring/*.ts` and
/// `src/client-sdk` are generated ForkLaunch workspace libraries the app does
/// not import; generated SDKs are derived from the contracts.)
fn application_sources() -> Vec<(String, String)> {
    let mut sources = Vec::new();
    for dir in ["src/api", "src/extraction", "src/shared"] {
        sources.extend(
            files(dir, &[".rs", ".toml", ".sql"])
                .into_iter()
                .filter(|(name, _)| !name.contains("/sdk/")),
        );
    }
    sources.extend(files("frontend/src", &[".ts", ".tsx"]));
    sources.extend(
        files("frontend", &[".conf", ".json", ".html"])
            .into_iter()
            .filter(|(name, _)| !name.contains("package-lock")),
    );
    sources.extend(files("scripts", &[".sh", ".py"]));
    sources.extend(files(".github", &[".yml", ".yaml", ".mjs", ".json"]));
    for file in [
        "docker-compose.yaml",
        "compose.backend.env",
        "Dockerfile.rust",
        "frontend/Dockerfile",
        ".env.example",
        "src/extraction/.env.example",
    ] {
        sources.push((file.to_owned(), read(file)));
    }
    assert!(
        sources.len() > 80,
        "found only {} source files; is the scan still looking?",
        sources.len()
    );
    sources
}

/// The image references of a Dockerfile (`FROM`) or Compose file (`image:`).
fn images(path: &str) -> Vec<String> {
    read(path)
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("FROM ").or_else(|| line.strip_prefix("image: ")))
        .map(|rest| rest.split_whitespace().next().unwrap_or("").to_owned())
        .collect()
}

#[test]
fn docker_base_images_are_pinned_to_a_version() {
    let mut checked = 0;
    for file in
        ["Dockerfile.rust", "Dockerfile.typescript", "frontend/Dockerfile", "docker-compose.yaml"]
    {
        for image in images(file) {
            // Stage names (`FROM builder`) and the two images this repository builds.
            if !image.contains(':')
                && !image.contains('@')
                && !image.contains('/')
                && file != "docker-compose.yaml"
            {
                assert!(
                    ["builder", "build", "runtime", "deps", "base"]
                        .iter()
                        .any(|stage| image.starts_with(stage)),
                    "{file}: `{image}` has no tag"
                );
                continue;
            }
            if image.starts_with("loandesk-") {
                assert!(
                    image.contains("${LOANDESK_VERSION"),
                    "{file}: `{image}` is not tagged with the release version"
                );
                continue;
            }
            checked += 1;
            if image.contains("@sha256:") {
                continue;
            }
            let tag = image
                .rsplit_once(':')
                .map(|(_, tag)| tag)
                .unwrap_or_else(|| panic!("{file}: `{image}` has no tag"));
            assert_ne!(tag, "latest", "{file}: `{image}`");
            // A full version: at least major.minor.patch (or major.minor for
            // distributions that number releases that way), not a moving
            // alias such as `22-alpine` or `bookworm-slim`.
            let version = tag.split(['-', '_']).next().unwrap_or("");
            let numbers: Vec<&str> = version.split('.').collect();
            assert!(
                numbers.len() >= 2
                    && numbers
                        .iter()
                        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())),
                "{file}: `{image}` is not pinned to a version"
            );
        }
    }
    assert!(checked >= 12, "checked only {checked} images");
}

#[test]
fn dependencies_are_pinned_by_a_committed_lockfile() {
    // One lockfile per package manager in use, all committed.
    for lockfile in
        ["Cargo.lock", "frontend/package-lock.json", "pnpm-lock.yaml", "src/pnpm-lock.yaml"]
    {
        assert!(exists(lockfile), "{lockfile} is missing");
    }
    let ignored = read(".gitignore");
    assert!(
        !ignored.contains("Cargo.lock")
            && !ignored.contains("package-lock")
            && !ignored.contains("pnpm-lock")
    );
    // Builds and CI install exactly what the lockfiles say.
    assert!(read("Dockerfile.rust").contains("cargo build --locked"));
    assert!(read("frontend/Dockerfile").contains("npm ci"));
    let ci = read(".github/workflows/ci.yml") + &read(".github/workflows/checks.yml");
    for frozen in ["cargo test --workspace --locked", "pnpm install --frozen-lockfile", "npm ci"] {
        assert!(ci.contains(frozen), "CI no longer runs `{frozen}`");
    }
    // The front end names exact versions: no ranges, tags or wildcards.
    let package: serde_json::Value =
        serde_json::from_str(&read("frontend/package.json")).expect("package.json");
    for section in ["dependencies", "devDependencies"] {
        for (name, version) in package[section].as_object().expect("dependency section") {
            let version = version.as_str().expect("version string");
            assert!(
                version.bytes().next().is_some_and(|b| b.is_ascii_digit())
                    && !version.contains(['*', 'x', ' ']),
                "frontend: {name} is `{version}`, not an exact version"
            );
        }
    }
    // The ForkLaunch crates are pinned exactly in every manifest.
    for manifest in ["src/api/Cargo.toml", "src/extraction/Cargo.toml"] {
        for line in read(manifest).lines().filter(|line| line.starts_with("forklaunch-")) {
            assert!(line.contains("\"=0."), "{manifest}: `{line}` is not pinned exactly");
        }
    }
}

#[test]
fn development_tools_are_not_production_dependencies() {
    // The front end ships two runtime libraries; compilers, bundlers and type
    // packages are devDependencies, and the image is built from the bundle only.
    let package: serde_json::Value =
        serde_json::from_str(&read("frontend/package.json")).expect("package.json");
    let runtime: Vec<&String> =
        package["dependencies"].as_object().expect("dependencies").keys().collect();
    assert_eq!(runtime, ["react", "react-dom"]);
    for tool in ["typescript", "vite", "@vitejs/plugin-react", "@types/react", "@types/react-dom"] {
        assert!(package["devDependencies"].get(tool).is_some(), "{tool} must be a devDependency");
    }
    let dockerfile = read("frontend/Dockerfile");
    assert!(
        dockerfile.contains("COPY --from=build /app/dist /usr/share/nginx/html"),
        "the web image carries only the built bundle"
    );
    assert!(!dockerfile.split("FROM ").last().unwrap_or("").contains("node_modules"));
    // The API also carries that bundle, for deployments with no web server
    // in front of it (docs/hosting.md). The Rust image has no Node, so the
    // bundle is committed and CI checks it is the build of the current source.
    assert!(package["scripts"]["build:embed"].as_str().is_some_and(|script| {
        script.contains("tsc --noEmit") && script.contains("--outDir ../src/api/web-dist")
    }));
    assert!(exists("src/api/web-dist/index.html"), "run `npm run build:embed` in frontend/");
    assert!(read("src/api/api/web.rs").contains("include_dir!(\"$CARGO_MANIFEST_DIR/web-dist\")"));
    let checks = read(".github/workflows/checks.yml");
    assert!(
        checks.contains("npm run build:embed")
            && checks.contains("git diff --exit-code -- ../src/api/web-dist"),
        "CI no longer checks that the embedded front end is current"
    );
    assert!(!read("Dockerfile.rust").to_lowercase().contains("npm "), "the Rust image has no Node");
    // The Rust image copies two binaries into the runtime stage: no toolchain,
    // no test or build crates.
    let rust = read("Dockerfile.rust");
    let runtime_stage = rust.split("AS runtime").nth(1).expect("runtime stage");
    assert!(
        runtime_stage.contains("COPY --from=builder /app/out/api")
            && !runtime_stage.contains("cargo")
    );
    for manifest in ["src/api/Cargo.toml", "src/extraction/Cargo.toml", "src/shared/Cargo.toml"] {
        let text = read(manifest);
        let dependencies = text.split("[dev-dependencies").next().unwrap_or("");
        for test_only in ["tracing-subscriber", "tower ="] {
            let in_runtime = dependencies
                .split("[dependencies")
                .skip(1)
                .any(|section| section.split("\n[").next().unwrap_or("").contains(test_only));
            assert!(
                !in_runtime,
                "{manifest}: `{test_only}` is a test tool and belongs in dev-dependencies"
            );
        }
    }
}

/// name -> the version requirement each manifest states for it.
fn cargo_requirements() -> BTreeMap<String, BTreeSet<String>> {
    let mut requirements: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (file, text) in files("src", &["Cargo.toml"]) {
        if file.contains("/sdk/") {
            continue;
        }
        // Only dependency tables: `[dependencies]`, `[dev-dependencies]`,
        // `[build-dependencies]` and their `[dependencies.<name>]` forms.
        let mut in_dependencies = false;
        let mut table: Option<String> = None;
        for line in text.lines().map(str::trim) {
            if let Some(header) = line.strip_prefix('[') {
                let header = header.trim_end_matches(']');
                let section = header.split('.').next().unwrap_or("");
                in_dependencies =
                    ["dependencies", "dev-dependencies", "build-dependencies"].contains(&section);
                table = header
                    .split_once('.')
                    .filter(|_| in_dependencies)
                    .map(|(_, name)| name.to_owned());
                continue;
            }
            if !in_dependencies {
                continue;
            }
            let version_of = |value: &str| value.split('"').nth(1).map(str::to_owned);
            if let Some(name) = &table {
                if let Some(version) = line.strip_prefix("version = ").and_then(version_of) {
                    requirements.entry(name.clone()).or_default().insert(version);
                }
            } else if let Some((name, value)) = line.split_once(" = ") {
                let version = if value.starts_with('"') {
                    version_of(value)
                } else {
                    value.split("version = ").nth(1).and_then(version_of)
                };
                if let Some(version) = version {
                    requirements.entry(name.trim().to_owned()).or_default().insert(version);
                }
            }
        }
    }
    requirements
}

#[test]
fn each_shared_library_is_on_one_version_across_packages() {
    // Rust: every crate that more than one package depends on is requested at
    // the same version everywhere, so the workspace builds one copy of it.
    let requirements = cargo_requirements();
    for shared in [
        "forklaunch-core",
        "forklaunch-infrastructure-s3",
        "forklaunch-worker-interface",
        "sqlx",
        "uuid",
        "tokio",
        "tracing",
        "opentelemetry",
        "serde_json",
        "axum",
    ] {
        let versions = requirements
            .get(shared)
            .unwrap_or_else(|| panic!("{shared} is not a dependency any more"));
        assert_eq!(versions.len(), 1, "{shared} is requested at {versions:?}");
    }
    for (name, versions) in &requirements {
        assert_eq!(versions.len(), 1, "`{name}` is requested at different versions: {versions:?}");
    }
    // And the lockfile resolves each framework crate to exactly one version.
    let lock = read("Cargo.lock");
    for package in [
        "forklaunch-core",
        "forklaunch-axum",
        "forklaunch-infrastructure-s3",
        "forklaunch-worker-database",
        "sqlx",
        "tokio",
        "axum",
    ] {
        let count = lock.matches(&format!("name = \"{package}\"\n")).count();
        assert_eq!(count, 1, "Cargo.lock holds {count} versions of {package}");
    }
    // JavaScript: the front end is one package with exact versions (above).
    // The generated ForkLaunch workspace under src/ aligns its packages through
    // `overrides` in pnpm-workspace.yaml: every @forklaunch package and the
    // shared third-party ones are forced to one version across the workspace.
    for workspace in ["pnpm-workspace.yaml", "src/pnpm-workspace.yaml"] {
        let text = read(workspace);
        let overrides = text
            .split("overrides:")
            .nth(1)
            .unwrap_or_else(|| panic!("{workspace} has no overrides"));
        for aligned in ["'@forklaunch/core':", "'@forklaunch/common':", "'@forklaunch/validator':"]
        {
            assert!(overrides.contains(aligned), "{workspace} no longer aligns {aligned}");
        }
    }
}

#[test]
fn no_secrets_are_hardcoded_in_source() {
    // Shapes of real credentials. None may appear in the application's code,
    // configuration, scripts or CI files.
    let mut findings = Vec::new();
    for (file, text) in application_sources() {
        if file.ends_with("__test__/controls.rs") {
            continue;
        }
        for (number, line) in text.lines().enumerate() {
            let lower = line.to_lowercase();
            let private_key = line.contains("-----BEGIN") && line.contains("PRIVATE KEY");
            let aws_key = line.split(|c: char| !c.is_ascii_alphanumeric()).any(|word| {
                word.len() == 20
                    && (word.starts_with("AKIA") || word.starts_with("ASIA"))
                    && word.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            });
            // A long literal assigned to something named like a credential.
            let named =
                ["password", "secret", "token", "api_key", "apikey", "key_hex", "access_key"]
                    .iter()
                    .any(|name| lower.contains(name));
            let literal = line.split(['"', '\'']).enumerate().any(|(index, part)| {
                index % 2 == 1
                    && part.len() >= 24
                    && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"+/=_-".contains(&b))
                    && part.bytes().any(|b| b.is_ascii_digit())
                    && part.bytes().any(|b| b.is_ascii_alphabetic())
            });
            let assigned = named && literal && (line.contains('=') || line.contains(": "));
            if private_key || aws_key || assigned {
                findings.push(format!("{file}:{}: {}", number + 1, line.trim()));
            }
        }
    }
    assert!(findings.is_empty(), "possible hardcoded secrets:\n{}", findings.join("\n"));

    // Where the secrets do come from: generated on first start for the local
    // stack, and from the environment everywhere.
    let compose = read("docker-compose.yaml");
    assert!(read("scripts/demo-secrets.sh").contains("/dev/urandom"));
    for from_file in [
        "POSTGRES_PASSWORD_FILE: /run/demo-secrets/postgres_password",
        "cat /run/demo-secrets/s3_secret_key",
    ] {
        assert!(compose.contains(from_file));
    }
    assert!(
        !compose.contains("ENCRYPTION_KEY_HEX:"),
        "the encryption key is never in the Compose file"
    );
    for unset in
        ["ENCRYPTION_KEY_HEX=", "S3_SECRET_ACCESS_KEY=", "DATABASE_URL=", "WORKER_CONTROL_TOKEN="]
    {
        let line = read(".env.example")
            .lines()
            .find(|line| line.starts_with(unset))
            .map(str::to_owned)
            .unwrap_or_default();
        assert!(
            line[unset.len()..].trim_start().starts_with('#') || line.len() == unset.len(),
            ".env.example sets {unset}"
        );
    }
    // Two things in the repository look like credentials and are not secrets;
    // both are local-only and refused or absent in production:
    // - DEMO_PASSWORD's default, the documented password of the seeded demo
    //   logins (README). Seeding is refused when APP_ENV=production.
    // - `minioadmin`, the MinIO image's public default, in the optional
    //   `observability` profile's telemetry store (generated ForkLaunch
    //   monitoring configs). That store holds traces, metrics and logs only.
    assert!(compose.contains("DEMO_PASSWORD: ${DEMO_PASSWORD:-loandesk-demo-2026}"));
    assert!(read("src/api/config.rs").contains("demo logins would be seeded"));
    // The public demo copy keeps demo logins but never that password: the
    // constant in config.rs exists only to refuse it (docs/hosting.md).
    let config = read("src/api/config.rs");
    assert!(config.contains("pub const PUBLIC_DEMO_PASSWORD: &str = \"loandesk-demo-2026\";"));
    assert!(config.contains("DEMO_PASSWORD is the public password from this repository"));
    assert!(!compose.contains("HOSTED_DEMO"), "the local stack is never a hosted demo");
    let hosting = read("docs/hosting.md");
    for setting in ["HOSTED_DEMO", "DEMO_PASSWORD", "PUBLIC_ORIGIN", "DATABASE_URL", "S3_BUCKET"] {
        assert!(hosting.contains(setting), "docs/hosting.md lost {setting}");
    }
    // A secret scan runs on every pull request, over the whole history.
    let checks = read(".github/workflows/checks.yml");
    assert!(checks.contains("ghcr.io/gitleaks/gitleaks:") && checks.contains("fetch-depth: 0"));
}

#[test]
fn a_release_sees_one_module_one_bucket_and_no_local_settings() {
    // The platform pairs `<m>-service` with `<m>-worker`: one database, one
    // pair of roles, one migration (docs/hosting.md). The CLI names a worker
    // project's component `<project>-worker`.
    let manifest = read(".forklaunch/manifest.toml");
    assert!(manifest.contains("name = \"loans-service\"") && manifest.contains("name = \"loans\""));
    assert!(
        manifest.contains("crateName = \"api\"") && manifest.contains("crateName = \"extraction\"")
    );
    assert_eq!(manifest.matches("object_store = \"s3\"").count(), 1, "one store, declared once");
    // The CLI reads each matching Compose service's own `environment` as what
    // a deployment needs. Local-only values live in the shared file instead.
    let compose = read("docker-compose.yaml");
    let block = |service: &str| {
        let start =
            compose.find(&format!("\n  {service}:\n")).unwrap_or_else(|| panic!("{service}"));
        // Up to the next line at the services' own indentation.
        compose[start + 1..]
            .lines()
            .skip(1)
            .take_while(|line| line.is_empty() || line.starts_with("    "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let (service, worker) = (block("loans-service"), block("loans-worker"));
    for (name, own) in [("loans-service", &service), ("loans-worker", &worker)] {
        let settings: Vec<&str> = own
            .lines()
            .filter(|line| line.starts_with("      ") && !line.trim_start().starts_with('#'))
            .map(str::trim)
            .collect();
        for local_only in [
            "<<:",
            "DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT:",
            "WORKER_ALLOW_PLAINTEXT_DEVELOPMENT:",
            "S3_URL:",
            "S3_PUBLIC_URL:",
            "APP_ENV:",
            "COOKIE_SECURE:",
            "HOSTED_DEMO:",
        ] {
            assert!(
                !settings.iter().any(|line| line.starts_with(local_only)),
                "{name} would put `{local_only}` into a release"
            );
        }
    }
    // The worker names the bucket and the API does not: that is what makes a
    // release record both as users of the one store.
    assert!(worker.contains("S3_BUCKET:") && !service.contains("S3_BUCKET:"));
    let shared = read("compose.backend.env");
    for setting in [
        "DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT=true",
        "S3_URL=http://object-store:9000",
        "S3_BUCKET=",
    ] {
        assert!(shared.contains(setting), "compose.backend.env lost {setting}");
    }
    assert!(compose.contains("- ./compose.backend.env"));
    // The module migrates once, in the worker's task.
    assert!(
        read("src/extraction/worker.rs").contains("api::persistence::database::migrate().await?")
    );
    assert!(read("src/api/persistence/repository.rs").contains("wait_for_schema"));
}

#[test]
fn tls_certificate_checks_are_never_turned_off() {
    let bypasses = [
        "danger_accept_invalid_certs",
        "danger_accept_invalid_hostnames",
        "rejectUnauthorized",
        "NODE_TLS_REJECT_UNAUTHORIZED",
        "InsecureSkipVerify",
        "verify=False",
        "verify_mode = ssl.CERT_NONE",
        "_create_unverified_context",
        "sslmode=disable",
        "sslmode=allow",
        "sslmode=prefer",
        "--insecure",
        "curl -k",
        "ssl_verify off",
        "proxy_ssl_verify off",
        "tls_insecure",
    ];
    for (file, text) in application_sources() {
        if file.ends_with("__test__/controls.rs") {
            continue;
        }
        for bypass in bypasses {
            assert!(!text.contains(bypass), "{file} contains `{bypass}`");
        }
    }
    // The database is the one connection that can be plaintext, and only when
    // an explicit local-development switch is on; production refuses it.
    let config = read("src/api/config.rs");
    assert!(config.contains("DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT must be false"));
    assert!(config.contains("S3_URL must be an https:// endpoint"));
    assert!(read("docs/deployment.md").contains("sslmode=verify-full"));
    // A hosted demo refuses the same plaintext allowances, and unset means off.
    assert!(config.contains(
        "let mode = if self.production { \"in production\" } else { \"as a hosted demo\" };"
    ));
    assert!(
        read("src/api/persistence/database.rs")
            .contains("match allow.as_deref().unwrap_or(\"false\")")
    );
    assert!(read("docs/hosting.md").contains("sslmode=verify-full"));
    // The runtime image trusts the public CAs and the database provider's CA.
    let image = read("Dockerfile.rust");
    assert!(image.contains("ca-certificates") && image.contains("rds-global-bundle.pem"));
}

#[test]
fn tests_are_not_skipped_wholesale() {
    for (file, text) in files("src/api", &[".rs"])
        .into_iter()
        .chain(files("src/extraction", &[".rs"]))
        .chain(files("src/shared", &[".rs"]))
    {
        if file.ends_with("__test__/controls.rs") || file.contains("/sdk/") {
            continue;
        }
        assert!(!text.contains("#[ignore"), "{file} has an ignored test");
        assert!(!text.contains("#[cfg(any())]"), "{file} compiles code out");
    }
    for (file, text) in files("scripts", &[".py", ".sh"]) {
        for skip in ["unittest.skip", "pytest.mark.skip", "SKIP=", "|| true  # test"] {
            assert!(!text.contains(skip), "{file} skips checks (`{skip}`)");
        }
    }
    // CI runs every suite, and none is allowed to fail.
    let ci = read(".github/workflows/ci.yml") + &read(".github/workflows/checks.yml");
    assert!(!ci.contains("continue-on-error"), "a CI job is allowed to fail");
    for suite in [
        "cargo test --workspace --locked",
        "python3 scripts/e2e.py",
        "sh scripts/verify-storage.sh",
        "sh scripts/verify-operations.sh",
        "python3 scripts/verify-lifecycle.py",
    ] {
        assert!(ci.contains(suite), "CI no longer runs `{suite}`");
    }
}

#[test]
fn tests_exercise_the_real_database_and_service_boundaries() {
    // The end-to-end job starts PostgreSQL, the object store, the API, the
    // worker and the web server, and drives them over HTTP.
    let checks = read(".github/workflows/checks.yml");
    assert!(checks.contains("docker compose up --build -d --wait"));
    let e2e = read("scripts/e2e.py");
    assert!(e2e.contains("urllib.request") && !e2e.contains("mock"));
    for boundary in
        ["/api/applications", "/api/documents/", "/api/auth/login", "/api/users", "/api/audit-log"]
    {
        assert!(e2e.contains(boundary), "scripts/e2e.py no longer calls {boundary}");
    }
    // The storage checks query the database directly.
    assert!(read("scripts/verify-storage.sh").contains("psql"));
    assert!(read("scripts/verify-lifecycle.py").contains("--lifecycle-once"));
}

#[test]
fn coverage_is_measured() {
    let checks = read(".github/workflows/checks.yml");
    assert!(checks.contains("cargo llvm-cov"), "CI no longer measures coverage");
    assert!(checks.contains("--fail-under-lines"), "coverage has no minimum");
}

#[test]
fn branch_protection_is_declared_as_code() {
    let ruleset: serde_json::Value =
        serde_json::from_str(&read(".github/rulesets/main.json")).expect("ruleset is JSON");
    assert_eq!(ruleset["enforcement"], "active");
    assert_eq!(ruleset["bypass_actors"], serde_json::json!([]), "nobody bypasses the rules");
    let rules = ruleset["rules"].as_array().expect("rules");
    let rule = |kind: &str| {
        rules.iter().find(|rule| rule["type"] == kind).unwrap_or_else(|| panic!("no `{kind}` rule"))
    };
    let review = &rule("pull_request")["parameters"];
    assert!(review["required_approving_review_count"].as_u64().is_some_and(|count| count >= 1));
    assert_eq!(review["require_code_owner_review"], true);
    assert_eq!(review["dismiss_stale_reviews_on_push"], true);
    rule("non_fast_forward");
    rule("deletion");
    // Every required check is a job that exists, under exactly that name.
    let ci = read(".github/workflows/ci.yml") + &read(".github/workflows/checks.yml");
    let required = rule("required_status_checks")["parameters"]["required_status_checks"]
        .as_array()
        .expect("checks")
        .clone();
    assert!(required.len() >= 8);
    for check in &required {
        let name = check["context"].as_str().expect("check name");
        assert!(
            ci.contains(&format!("    name: {name}\n")),
            "required check `{name}` is not a CI job"
        );
    }
    // And every CI job is required.
    let jobs = ci.lines().filter(|line| line.starts_with("    name: ")).count();
    assert_eq!(jobs, required.len(), "a CI job is not a required check (or the reverse)");
    assert!(
        exists("scripts/apply-branch-protection.sh")
            && exists(".github/CODEOWNERS")
            && exists(".github/pull_request_template.md")
    );
}

#[test]
fn pull_requests_follow_a_template_that_asks_for_testing_and_risk() {
    let template = read(".github/pull_request_template.md");
    for section in
        ["## What changed", "## How it was tested", "## Risk", "## Rollback", "## Changelog"]
    {
        assert!(template.contains(section), "the pull request template lost `{section}`");
    }
    // Commits are traceable too: a convention, checked in CI and by a hook.
    assert!(
        exists("scripts/check-commit-message.sh")
            && read(".husky/commit-msg").contains("check-commit-message")
    );
}

/// (alert name, severity, expression) of every alert rule.
fn alerts() -> Vec<(String, String, String)> {
    let rules = read("src/monitoring/alerts/loandesk.rules.yaml");
    let mut alerts = Vec::new();
    for block in rules.split("- alert: ").skip(1) {
        let block = block.split("\n  - name:").next().unwrap_or(block);
        let name = block.lines().next().unwrap_or("").trim().to_owned();
        let severity = block
            .split("severity: ")
            .nth(1)
            .and_then(|rest| rest.split([',', ' ', '}', '\n']).next())
            .unwrap_or("")
            .to_owned();
        assert!(block.contains("summary: "), "alert {name} has no summary");
        assert!(block.contains("runbook: "), "alert {name} has no runbook");
        alerts.push((name, severity, block.to_owned()));
    }
    assert!(alerts.len() >= 25, "found only {} alerts", alerts.len());
    alerts
}

#[test]
fn alerts_are_routed_to_a_person() {
    let routing = read("src/monitoring/alerts/alertmanager.yaml");
    let config = routing
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    // Every severity an alert can have has a route, and every route a receiver.
    let severities: BTreeSet<String> =
        alerts().into_iter().map(|(_, severity, _)| severity).collect();
    assert_eq!(severities.into_iter().collect::<Vec<_>>(), ["page", "ticket"]);
    for (severity, receiver) in [("page", "on-call"), ("ticket", "team")] {
        assert!(
            config.contains(&format!(
                "matchers: [severity = \"{severity}\"]\n      receiver: {receiver}"
            )),
            "{severity} alerts are not routed"
        );
        assert!(
            config.contains(&format!("  - name: {receiver}\n")),
            "receiver {receiver} is not defined"
        );
    }
    assert!(
        config.contains("route:\n  receiver: team"),
        "alerts without a known severity must still reach someone"
    );
    // Destinations come from files mounted at deploy time, never from the repository.
    assert!(config.contains("routing_key_file:") && config.contains("url_file:"));
    assert!(
        !config.contains("routing_key:")
            && !config.contains("api_url:")
            && !config.contains("https://hooks.")
    );
    // Every runbook an alert points to exists, with that heading.
    for (name, _, block) in alerts() {
        let target = block
            .split("runbook: ")
            .nth(1)
            .and_then(|rest| rest.lines().next())
            .unwrap_or("")
            .trim();
        let (file, anchor) = target.split_once('#').unwrap_or((target, ""));
        let text = read(file);
        let headings: Vec<String> = text
            .lines()
            .filter(|line| line.starts_with('#'))
            .map(|line| {
                line.trim_start_matches('#')
                    .trim()
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(|c: char| !(c.is_ascii_alphanumeric() || c == '-'), "")
            })
            .collect();
        assert!(
            anchor.is_empty() || headings.iter().any(|heading| heading == anchor),
            "alert {name}: {file} has no section `{anchor}`"
        );
    }
}

#[test]
fn security_relevant_events_are_monitored_and_alertable() {
    // Every event name an alert counts is written by the code (an alert on an
    // event nothing emits would never fire), and every security event the
    // code writes has an alert.
    let code: String = files("src/api", &[".rs"])
        .into_iter()
        .chain(files("src/extraction", &[".rs"]))
        .filter(|(name, _)| !name.contains("__test__"))
        .map(|(_, text)| text)
        .collect();
    let rules = read("src/monitoring/alerts/loandesk.rules.yaml");
    let mut counted = BTreeSet::new();
    for expression in rules.split("event=").skip(1) {
        let quoted = expression.trim_start_matches('~').split('"').nth(1).unwrap_or("");
        counted.extend(quoted.split('|').map(str::to_owned));
    }
    for event in &counted {
        assert!(
            code.contains(&format!("\"{event}\"")),
            "an alert counts `{event}`, which the code never logs"
        );
    }
    for event in [
        "login_failed",
        "login_rate_limited",
        "login_refused_disabled",
        "invalid_session",
        "rate_limited",
        "permission_denied",
        "cross_tenant_storage_key",
        "password_change_refused",
        "ssn_revealed",
        "application_exported",
        "document_downloaded",
        "user_created",
        "user_role_changed",
        "user_disabled",
        "user_enabled",
        "access_review_overdue",
        "stale_accounts_disabled",
        "stale_admin_kept",
        "encryption_key_rotation_due",
        "lifecycle_step_failed",
        "lifecycle_pass",
        "panic",
        "dependency_error",
        "browser_error",
    ] {
        assert!(counted.contains(event), "no alert rule counts the `{event}` event");
    }
}

#[test]
fn dashboards_are_defined_as_code() {
    let dashboards = files("src/monitoring/grafana-provisioning/dashboards", &[".json"]);
    assert!(dashboards.len() >= 2, "expected the provisioned dashboards");
    for (file, text) in &dashboards {
        let dashboard: serde_json::Value =
            serde_json::from_str(text).unwrap_or_else(|_| panic!("{file} is JSON"));
        assert!(
            dashboard["panels"].as_array().is_some_and(|panels| !panels.is_empty()),
            "{file} has no panels"
        );
    }
    // Grafana loads them from the repository, not from its database.
    assert!(exists("src/monitoring/grafana-provisioning/dashboards/default.yaml"));
    assert!(read("docker-compose.yaml").contains(
        "./src/monitoring/grafana-provisioning/dashboards:/etc/grafana/provisioning/dashboards"
    ));
}

#[test]
fn queries_select_what_they_need() {
    for (file, text) in
        files("src/api", &[".rs"]).into_iter().chain(files("src/extraction", &[".rs"]))
    {
        if file.contains("__test__") || file.contains("/sdk/") {
            continue;
        }
        let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for wide in ["SELECT *", "RETURNING *", "select *", ".* FROM"] {
            assert!(!flat.contains(wide), "{file} runs `{wide}`: name the columns");
        }
    }
    // List pages never read or decrypt the free text or the owners; the
    // single-application view does.
    let service = read("src/api/domain/services/applications.rs");
    let list = service
        .split("pub async fn list")
        .nth(1)
        .and_then(|rest| rest.split("pub async fn create").next())
        .expect("list function");
    assert!(
        !list.contains("reveal")
            && !list.contains("FROM owners")
            && !list.contains("FROM documents")
    );
}

#[test]
fn scheduled_jobs_run_once_not_once_per_instance_and_cannot_overlap() {
    let lifecycle = read("src/extraction/lifecycle.rs");
    // One pass holds a session-level advisory lock; a second instance, or a
    // tick that comes due during a long pass, does not wait for it.
    assert!(lifecycle.contains("SELECT pg_try_advisory_lock($1, $2)"));
    assert!(lifecycle.contains("totals.skipped = true"));
    assert!(lifecycle.contains("MissedTickBehavior::Skip"));
    assert!(lifecycle.contains("SELECT pg_advisory_unlock($1, $2)"));
    assert!(
        !lifecycle.contains("pg_advisory_lock("),
        "the job must never block waiting for the lock"
    );
    // Seeding takes a transaction lock for the same reason.
    assert!(read("src/api/seed.rs").contains("pg_advisory_xact_lock"));
    // scripts/verify-lifecycle.py holds the lock from another session and
    // checks that a pass reports `skipped`.
    assert!(read("scripts/verify-lifecycle.py").contains("pg_advisory_lock(4107, 2)"));
}

#[test]
fn this_product_has_no_payments_webhooks_realtime_connections_or_caches() {
    // Requirements about card data, payment keys, payment network isolation,
    // webhooks (incoming and outgoing), consent for optional processing,
    // real-time connections and cache invalidation do not apply to LoanDesk:
    // it has none of those. This test fails the day one is added, so the
    // statement in docs/controls.md cannot go stale silently.
    let registry = api::sdk::registry();
    for route in registry.routes() {
        let path = route.path.to_lowercase();
        for absent in [
            "webhook",
            "payment",
            "card",
            "checkout",
            "billing",
            "subscribe",
            "consent",
            "ws",
            "socket",
            "stream",
        ] {
            assert!(
                !path
                    .split('/')
                    .any(|segment| segment == absent || segment.starts_with(&format!("{absent}s"))),
                "{} looks like a {absent} route",
                route.path
            );
        }
    }
    let manifests = read("src/api/Cargo.toml")
        + &read("src/extraction/Cargo.toml")
        + &read("src/shared/Cargo.toml")
        + &read("frontend/package.json");
    for absent in [
        "stripe",
        "braintree",
        "adyen",
        "paypal",
        "tungstenite",
        "socket.io",
        "moka",
        "memcache",
        "lettre",
        "sendgrid",
        "twilio",
        "reqwest",
    ] {
        assert!(
            !manifests.to_lowercase().contains(absent),
            "a dependency on `{absent}` was added: revisit docs/controls.md"
        );
    }
    // No stored model has a card, payment or consent field.
    for entity in api::persistence::compliance_schema::declarations() {
        for field in entity.fields {
            for absent in ["card", "pan", "cvv", "payment", "consent", "marketing"] {
                assert!(
                    !field.storage_name.split('_').any(|part| part == absent),
                    "{}.{} looks like {absent} data",
                    entity.storage,
                    field.storage_name
                );
            }
        }
        let levels =
            format!("{:?}", entity.fields.iter().map(|field| field.level).collect::<Vec<_>>());
        assert!(
            !levels.to_lowercase().contains("pci"),
            "{} classifies a field as card data",
            entity.storage
        );
    }
    // The API keeps no cache: every request reads PostgreSQL (the session,
    // the role and the record), so there is nothing to invalidate. The one
    // Redis client in the tree is the worker's optional pause/resume signal bus.
    let api_manifest = read("src/api/Cargo.toml");
    assert!(!api_manifest.contains("redis"));
    // The only outbound connections are PostgreSQL, the object store and the
    // OTLP collector (docs/data-sharing.md).
    assert!(exists("docs/data-sharing.md"));
}

#[test]
fn the_readme_and_architecture_documents_cover_what_they_must() {
    let readme = read("README.md");
    for part in [
        "## Run it",
        "docker compose up --build",
        "## Test it",
        "cargo test --workspace",
        "python3 scripts/e2e.py",
        "## Demo logins",
        "## How it fits together",
    ] {
        assert!(readme.contains(part), "the README lost `{part}`");
    }
    let architecture = read("docs/architecture.md");
    for part in [
        "| Web |",
        "| API |",
        "| Worker |",
        "| Database |",
        "| Object store |",
        "## Request path",
        "## Where each control is enforced",
    ] {
        assert!(architecture.contains(part), "docs/architecture.md lost `{part}`");
    }
    // Every file the controls page points at exists.
    let controls = read("docs/controls.md");
    let mut checked = 0;
    for reference in controls.split('`').skip(1).step_by(2) {
        let path = reference.split([' ', ':', '#', '(']).next().unwrap_or("");
        let looks_like_a_path = path.contains('/')
            && !path.starts_with('/')
            && !path.starts_with("http")
            && !path.contains('*')
            && !path.contains('<')
            && !path.contains('=')
            && path
                .rsplit('/')
                .next()
                .is_some_and(|name| name.contains('.') || path.ends_with('/'));
        if looks_like_a_path {
            checked += 1;
            assert!(exists(path), "docs/controls.md points at `{path}`, which does not exist");
        }
    }
    assert!(checked >= 60, "checked only {checked} file references in docs/controls.md");
}

#[test]
fn every_test_the_controls_page_names_exists() {
    let tests: String = files("src", &[".rs"]).into_iter().map(|(_, text)| text).collect();
    let scripts = read("scripts/e2e.py")
        + &read("scripts/verify-lifecycle.py")
        + &read("scripts/verify-storage.sh")
        + &read("scripts/verify-operations.sh")
        + &read("scripts/load-test.py");
    let controls = read("docs/controls.md");
    // A backticked snake_case name is a test, unless it is an event, audit
    // action or index name, which then has to appear in the code or migrations.
    let migrations: String =
        files("src/api/migrations", &[".sql"]).into_iter().map(|(_, sql)| sql).collect();
    let mut checked = 0;
    for reference in controls.split('`').skip(1).step_by(2) {
        let snake_case = reference.len() > 20
            && reference.contains('_')
            && reference.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !snake_case {
            continue;
        }
        if tests.contains(&format!("fn {reference}(")) {
            checked += 1;
        } else {
            assert!(
                tests.contains(&format!("\"{reference}\"")) || migrations.contains(reference),
                "docs/controls.md names `{reference}`, which is neither a test nor a name in the code"
            );
        }
    }
    assert!(checked >= 50, "checked only {checked} test names in docs/controls.md");
    // Script checks are quoted by their label.
    for label in controls.split("\u{201c}").skip(1) {
        let label = label.split('\u{201d}').next().unwrap_or("");
        assert!(
            scripts.contains(label),
            "docs/controls.md quotes the check \u{201c}{label}\u{201d}, which no script prints"
        );
    }
}
