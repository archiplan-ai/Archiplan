//! Statement results: applied / noop / error, cascade reports, query output
//! and check findings.

use std::fmt;

use crate::error::LangError;

/// The result of one successfully processed statement.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// The model changed.
    Applied,
    /// The statement restates something identical that already exists.
    Noop,
    /// A `delete` succeeded; `cascade` lists everything removed, rendered as
    /// statements in creation order.
    Deleted {
        /// Every removed element, rendered as the statement that created it.
        cascade: Vec<String>,
    },
    /// A query result: statements that recreate the requested slice.
    Statements(Vec<String>),
    /// A `check` result: model-completeness findings.
    Findings(Vec<Finding>),
    /// A braced statement (`X { ... }` or `node X { ... }`); carries the results
    /// of the inner statements. For `node X { ... }` the first entry reports the
    /// `node X` part itself.
    Block(Vec<StatementResult>),
}

impl Outcome {
    /// Whether this outcome (or, for blocks, any inner outcome) changed the model.
    pub fn changed_model(&self) -> bool {
        match self {
            Outcome::Applied | Outcome::Deleted { .. } => true,
            Outcome::Noop | Outcome::Statements(_) | Outcome::Findings(_) => false,
            Outcome::Block(inner) => inner.iter().any(|r| r.outcome.changed_model()),
        }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Applied => f.write_str("applied"),
            Outcome::Noop => f.write_str("noop"),
            Outcome::Deleted { cascade } => {
                f.write_str("deleted:")?;
                for s in cascade {
                    write!(f, "\n  {s}")?;
                }
                Ok(())
            }
            Outcome::Statements(lines) => {
                if lines.is_empty() {
                    return f.write_str("(empty)");
                }
                for (i, s) in lines.iter().enumerate() {
                    if i > 0 {
                        f.write_str("\n")?;
                    }
                    f.write_str(s)?;
                }
                Ok(())
            }
            Outcome::Findings(findings) => {
                if findings.is_empty() {
                    return f.write_str("no findings");
                }
                for (i, fi) in findings.iter().enumerate() {
                    if i > 0 {
                        f.write_str("\n")?;
                    }
                    write!(f, "{fi}")?;
                }
                Ok(())
            }
            Outcome::Block(inner) => {
                if inner.iter().any(|r| r.outcome.changed_model()) {
                    f.write_str("applied")
                } else {
                    f.write_str("noop")
                }
            }
        }
    }
}

/// One statement paired with its outcome.
#[derive(Clone, Debug)]
pub struct StatementResult {
    /// The statement source text, as submitted.
    pub source: String,
    /// What happened.
    pub outcome: Outcome,
}

/// A batch failed: the whole batch was rolled back.
#[derive(Clone, Debug)]
pub struct BatchError {
    /// Index of the failing top-level statement within the batch.
    pub index: usize,
    /// Why it failed.
    pub error: LangError,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "statement {} failed: {}", self.index, self.error)
    }
}

impl std::error::Error for BatchError {}

/// The result of one interactively applied statement.
#[derive(Clone, Debug)]
pub struct InteractiveResult {
    /// The statement source text, as submitted.
    pub source: String,
    /// Outcome, or the error that rejected this one statement.
    pub result: Result<Outcome, LangError>,
}

/// A model-completeness finding: a state that is legal mid-construction but
/// suspect. Findings are surfaced by `check`, never by rejecting writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    /// An edge whose shape conformance drifted after a classifier edge was removed.
    ShapeDrift {
        /// The nonconforming edge, rendered as a statement.
        statement: String,
        /// Which slot drifted: `source`, `target` or `carrier`.
        slot: String,
        /// The pattern the slot must match.
        expected: String,
        /// The node that no longer matches.
        actual: String,
    },
    /// Carried traffic on a delegated port that matches no delegation.
    UnroutedTraffic {
        /// The unroutable connection edge, rendered as a statement.
        statement: String,
        /// The delegated port the edge attaches to.
        port: String,
    },
    /// A delegated port with no attached connections.
    DelegatedPortWithoutConnections {
        /// The port, as `path.port`.
        port: String,
    },
    /// A view with no edges.
    EmptyView {
        /// The view name.
        view: String,
    },
    /// A type with no instances.
    TypeWithoutInstances {
        /// `"rel"` or `"conn"`.
        kind: &'static str,
        /// The type name.
        name: String,
    },
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::ShapeDrift {
                statement,
                slot,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "shape drift: {statement} — {slot} {actual} no longer matches {expected}"
                )
            }
            Finding::UnroutedTraffic { statement, port } => {
                write!(
                    f,
                    "unrouted traffic: {statement} — no delegation on {port} matches"
                )
            }
            Finding::DelegatedPortWithoutConnections { port } => {
                write!(f, "delegated port without connections: {port}")
            }
            Finding::EmptyView { view } => write!(f, "empty view: {view}"),
            Finding::TypeWithoutInstances { kind, name } => {
                write!(f, "type without instances: {kind} {name}")
            }
        }
    }
}
