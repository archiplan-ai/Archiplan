//! Presets: the standard library generalized to a statement batch.
//!
//! A preset is a named list of statements — relations, nodes and the edges
//! wiring them — loaded into a fresh model before any user statements
//! (`requirements/modeling-lang/ontology.md`). Everything a preset creates is
//! stdlib: dumps omit it, mutations that would alter or remove it are
//! rejected (`E_STDLIB_PROTECTED`), and analyses treat it as scaffolding
//! rather than model content.
//!
//! Every preset must define the classifier relation `type_of` with the exact
//! stdlib shape `rel trans type_of := * -> *` — the layer split
//! ([`crate::Model::layer_of`]) and the `types` query filter key off it; a
//! preset that omits it or defines it divergently is rejected at load
//! (`E_PRESET_INVALID`).

use serde_json::{Value, json};

use crate::error::{ErrorCode, LangError};
use crate::statement::{Statement, parse_statement};

/// A named statement batch loaded as the standard library of a model.
///
/// Presets hold creation statements only — `define`, `rel-edge`, `conn-edge`,
/// `app`. A preset defines what exists; it does not mutate or read.
#[derive(Clone, PartialEq, Debug)]
pub struct Preset {
    name: String,
    statements: Vec<Statement>,
}

impl Preset {
    /// The core preset: exactly the historical stdlib — `type_of`, nothing
    /// else. This is what [`crate::Workspace::new`] loads, and what models
    /// saved before presets existed are restored with.
    pub fn core() -> Self {
        Self::from_value(
            "core",
            &json!([
                { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
                  "source": "*", "target": "*" }
            ]),
        )
        .expect("the core preset is valid")
    }

    /// The default ontology preset of
    /// `requirements/modeling-lang/ontology.md`: `type_of` plus the ontology
    /// type nodes `Data`, `Service`, `Function` and `Storage`.
    pub fn default_ontology() -> Self {
        Self::from_value(
            "default",
            &json!([
                { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
                  "source": "*", "target": "*" },
                { "stmt": "define", "node": "Data" },
                { "stmt": "define", "node": "Service" },
                { "stmt": "define", "node": "Function" },
                { "stmt": "define", "node": "Storage" }
            ]),
        )
        .expect("the default ontology preset is valid")
    }

    /// Parse a preset from a JSON array of statement objects.
    pub fn from_value(name: &str, value: &Value) -> Result<Self, LangError> {
        let items = value.as_array().ok_or_else(|| {
            LangError::new(
                ErrorCode::PresetInvalid,
                format!("preset `{name}` is not a JSON array of statements"),
            )
        })?;
        let mut statements = Vec::with_capacity(items.len());
        for v in items {
            statements.push(parse_statement(v)?);
        }
        Self::new(name, statements)
    }

    /// Build a preset from parsed statements, enforcing the creation-only
    /// rule.
    pub fn new(name: &str, statements: Vec<Statement>) -> Result<Self, LangError> {
        for stmt in &statements {
            let creation = matches!(
                stmt,
                Statement::Define(_)
                    | Statement::RelEdge { .. }
                    | Statement::ConnEdge { .. }
                    | Statement::App { .. }
            );
            if !creation {
                return Err(LangError::new(
                    ErrorCode::PresetInvalid,
                    format!(
                        "a preset holds creation statements only; `{}` does not belong",
                        stmt.pseudo()
                    ),
                )
                .with_subject(stmt.to_value()));
            }
        }
        Ok(Preset {
            name: name.to_string(),
            statements,
        })
    }

    /// The preset's name, recorded on models built from it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The preset's statements.
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}
