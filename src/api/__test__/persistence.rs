//! Checks on data classification. They need no database: they compare the
//! stored models (the same ones the repository compiles against) with the
//! migrations.

use api::persistence::{
    compliance_schema::declarations,
    entities::{StoredApplication, StoredDocument, StoredOwner, StoredUserMfa},
};
use forklaunch_core::persistence::compliance::{ComplianceLevel, ComplianceMetadata};
use std::collections::{BTreeMap, BTreeSet};

/// Every migration, in order, as (file name, SQL). Read from the directory so
/// a new file is checked the moment it is added.
fn migrations() -> Vec<(String, String)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    let mut files: Vec<(String, String)> = std::fs::read_dir(dir)
        .expect("migrations directory")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "sql"))
        .map(|path| {
            let name = path.file_name().expect("file name").to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&path).expect("readable migration"))
        })
        .collect();
    files.sort();
    assert!(files.len() >= 14, "expected the 14 known migrations, found {}", files.len());
    files
}

/// SQL with `--` comments removed.
fn statements(sql: &str) -> Vec<String> {
    let code: String = sql
        .lines()
        .map(|line| line.split("--").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    code.split(';')
        .map(|statement| statement.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|statement| !statement.is_empty())
        .collect()
}

/// (index name, table, definition after the table name) of every index.
fn indexes() -> Vec<(String, String, String)> {
    let mut found = Vec::new();
    for (_, sql) in migrations() {
        for statement in statements(&sql) {
            let words: Vec<&str> = statement.split(' ').collect();
            if words[0] != "CREATE" || !words.contains(&"INDEX") {
                continue;
            }
            let on = words.iter().position(|word| *word == "ON").expect("index has a table");
            found.push((
                words[on - 1].to_owned(),
                words[on + 1].to_owned(),
                words[on + 2..].join(" "),
            ));
        }
    }
    found
}

/// Columns still present for rollback but emptied by `api --migrate`; they are
/// not part of the stored models (see persistence/legacy.rs).
const LEGACY_COLUMNS: [(&str, &str); 9] = [
    ("owners", "full_name"),
    ("owners", "ssn_encrypted"),
    ("owners", "ssn_last4"),
    ("owners", "date_of_birth"),
    ("applications", "purpose"),
    ("applications", "decision_note"),
    ("documents", "filename"),
    ("documents", "attempts"),
    ("documents", "locked_at"),
];

/// table -> columns, read from CREATE TABLE and ALTER TABLE ... ADD COLUMN.
fn schema() -> BTreeMap<String, BTreeSet<String>> {
    let mut tables: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut creating = false;
    let all: Vec<String> = migrations().into_iter().map(|(_, sql)| sql).collect();
    for line in all.iter().flat_map(|sql| sql.lines()) {
        let line = line.trim();
        let words: Vec<&str> = line.split_whitespace().collect();
        if line.starts_with("CREATE TABLE") {
            current = Some(words[2].to_owned());
            creating = true;
            tables.entry(words[2].to_owned()).or_default();
        } else if line.starts_with("ALTER TABLE") {
            current = Some(words[2].to_owned());
            creating = false;
            if let Some(index) = words.iter().position(|word| *word == "COLUMN")
                && words.get(index.wrapping_sub(1)) == Some(&"ADD")
            {
                tables.entry(words[2].to_owned()).or_default().insert(words[index + 1].to_owned());
            }
        } else if creating && line.starts_with(')') {
            creating = false;
        } else if let Some(table) = &current {
            if creating
                && !line.is_empty()
                && !line.starts_with("--")
                && !line.starts_with("CONSTRAINT")
                && words[0].chars().all(|c| c.is_ascii_lowercase() || c == '_')
            {
                tables.entry(table.clone()).or_default().insert(words[0].to_owned());
            } else if !creating && line.starts_with("ADD COLUMN") {
                tables.entry(table.clone()).or_default().insert(words[2].to_owned());
            }
        }
        if line.ends_with(';') {
            creating = false;
        }
    }
    tables
}

#[test]
fn every_table_and_column_in_the_migrations_has_a_classification() {
    let declared: BTreeMap<&str, BTreeSet<&str>> = declarations()
        .iter()
        .map(|entity| {
            (entity.storage, entity.fields.iter().map(|field| field.storage_name).collect())
        })
        .collect();
    let schema = schema();
    assert!(schema.len() >= 7, "migration parser found only {:?}", schema.keys());
    for (table, columns) in &schema {
        let fields = declared
            .get(table.as_str())
            .unwrap_or_else(|| panic!("table `{table}` has no classified stored model"));
        for column in columns {
            let legacy = LEGACY_COLUMNS.contains(&(table.as_str(), column.as_str()));
            assert!(
                legacy || fields.contains(column.as_str()),
                "column `{table}.{column}` is not classified: add it to the stored model with a level"
            );
        }
    }
}

#[test]
fn personal_data_is_bound_to_encryption_not_just_labelled() {
    let protected = |members: Option<&'static [_]>| -> Vec<&'static str> {
        members
            .unwrap_or(&[])
            .iter()
            .map(|member: &forklaunch_core::persistence::compliance::ProtectedDeclaration| {
                member.member
            })
            .collect()
    };
    assert_eq!(
        protected(StoredOwner::PROTECTED_MEMBERS),
        ["full_name_ciphertext", "ssn_ciphertext", "date_of_birth_ciphertext"]
    );
    // Free-text fields can hold anything, so they are treated as personal data.
    assert_eq!(
        protected(StoredApplication::PROTECTED_MEMBERS),
        ["purpose_ciphertext", "decision_note_ciphertext"]
    );
    assert_eq!(protected(StoredDocument::PROTECTED_MEMBERS), ["filename_ciphertext"]);
    assert_eq!(protected(StoredUserMfa::PROTECTED_MEMBERS), ["secret_ciphertext"]);

    // In a persisted model the classification itself forces ciphertext: every
    // classified field must be one of the protected members above.
    for entity in declarations() {
        if let Some(members) = entity.protected_members {
            for field in entity.fields.iter().filter(|field| field.level != ComplianceLevel::None) {
                assert!(
                    members.iter().any(|member| member.member == field.rust_name),
                    "{}.{} is classified but not encrypted",
                    entity.entity,
                    field.rust_name
                );
            }
        }
    }
}

#[test]
fn no_stored_model_keeps_a_readable_ssn() {
    for entity in declarations() {
        for field in entity.fields {
            let name = field.storage_name;
            assert!(
                !name.contains("ssn") || name == "ssn_ciphertext",
                "{}.{name} would store SSN data outside the sealed field",
                entity.storage
            );
        }
    }
}

#[test]
fn indexes_on_existing_tables_are_built_without_blocking_writes() {
    let mut checked = 0;
    for (file, sql) in migrations() {
        let statements = statements(&sql);
        let created_here: Vec<&str> = statements
            .iter()
            .filter(|statement| statement.starts_with("CREATE TABLE "))
            .map(|statement| statement.split(' ').nth(2).expect("table name"))
            .collect();
        for statement in &statements {
            let words: Vec<&str> = statement.split(' ').collect();
            if words[0] != "CREATE" || !words.contains(&"INDEX") {
                continue;
            }
            let on = words.iter().position(|word| *word == "ON").expect("index has a table");
            if created_here.contains(&words[on + 1]) {
                continue; // A table created in the same migration is empty.
            }
            checked += 1;
            assert!(
                words.contains(&"CONCURRENTLY"),
                "{file}: `{statement}` locks writes on {}; use CREATE INDEX CONCURRENTLY",
                words[on + 1]
            );
            // CONCURRENTLY cannot run in a transaction, and sqlx sends a
            // migration as one query: opt out, and keep the statement alone.
            assert!(
                sql.starts_with("-- no-transaction"),
                "{file} must start with -- no-transaction"
            );
            assert_eq!(statements.len(), 1, "{file} must hold exactly one statement");
        }
    }
    assert_eq!(checked, 12, "expected the 12 indexes added to existing tables");
}

#[test]
fn every_filter_and_sort_the_services_run_has_an_index() {
    // (table, leading columns of an index, the query it serves). Row-level
    // security adds `tenant_id = ...` to every statement, hence the prefix.
    let needed = [
        ("applications", "(tenant_id, created_at DESC, id DESC)", "credit officers' list page"),
        (
            "applications",
            "(tenant_id, created_by, created_at DESC, id DESC)",
            "relationship managers' list page",
        ),
        ("applications", "(tenant_id, created_by, idempotency_key)", "idempotent create"),
        ("applications", "USING gin (search_tokens)", "search on encrypted text"),
        ("applications", "(created_by)", "foreign key, visibility filter"),
        ("applications", "(decided_by)", "foreign key"),
        ("owners", "(application_id)", "owners of an application"),
        ("documents", "(application_id)", "documents of an application"),
        ("documents", "(uploaded_by)", "foreign key"),
        ("sessions", "(user_id)", "ending a user's sessions"),
        ("sessions", "(expires_at)", "removing expired sessions"),
        ("users", "(tenant_id, full_name, id)", "user list page"),
        ("audit_log", "(user_id)", "audit trail by actor"),
        ("audit_log", "(entity_id)", "audit trail by record"),
        ("audit_log", "(tenant_id, at)", "audit trail by time"),
    ];
    let indexes = indexes();
    for (table, columns, purpose) in needed {
        assert!(
            indexes
                .iter()
                .any(|(_, on, definition)| on == table && definition.starts_with(columns)),
            "no index on {table} {columns} for: {purpose}"
        );
    }
    // Lookups by key use the primary keys and unique constraints of 0001:
    // sessions.token_hash, users.email, applications.reference, documents.storage_key.
    let init = &migrations()[0].1;
    for unique in ["token_hash TEXT PRIMARY KEY", "email TEXT NOT NULL UNIQUE"] {
        let flat = init.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(flat.contains(unique), "0001 no longer declares `{unique}`");
    }
}
