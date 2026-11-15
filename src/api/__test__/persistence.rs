//! Checks on data classification. They need no database: they compare the
//! stored models (the same ones the repository compiles against) with the
//! migrations.

use api::persistence::{
    compliance_schema::declarations,
    entities::{StoredApplication, StoredDocument, StoredOwner, StoredUserMfa},
};
use forklaunch_core::persistence::compliance::{ComplianceLevel, ComplianceMetadata};
use std::collections::{BTreeMap, BTreeSet};

const MIGRATIONS: [&str; 2] = [
    include_str!("../migrations/0001_init.sql"),
    include_str!("../migrations/0002_forklaunch.sql"),
];

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
    for line in MIGRATIONS.iter().flat_map(|sql| sql.lines()) {
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
