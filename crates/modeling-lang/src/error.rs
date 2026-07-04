//! The error contract: structured, machine-readable statement failures.
//!
//! Every rejected statement produces a [`LangError`] with a stable [`ErrorCode`],
//! the offending statement as submitted, the elements involved, the violated
//! constraint where applicable, and a suggested next step. Codes are append-only.

use std::fmt;

/// Stable error codes from the catalog in `requirements/modeling-lang/errors.md`.
///
/// `CrossScope` is appended by this implementation (the catalog is append-only):
/// it rejects a connection whose ends do not share a parent scope, and an
/// application whose inner node is not a direct child of the delegating node.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorCode {
    /// The statement does not parse; position and expected tokens included.
    Parse,
    /// A referenced node / type / view / path does not resolve in scope.
    UnknownName,
    /// Node creation or rename collides with a sibling's name.
    DupName,
    /// A rel / conn / view is redeclared with a different definition.
    Redeclared,
    /// An end or carrier fails the type's pattern at edge creation.
    ShapeViolation,
    /// A ternary connection is instantiated without a carrier.
    CarrierRequired,
    /// A binary connection is instantiated with a carrier.
    CarrierForbidden,
    /// A port is reused with a different connection type than its first use fixed.
    PortTypeConflict,
    /// A port of a directed type is reused on the opposite side.
    PortSideConflict,
    /// An application delegates a port no connection attaches to.
    NoOuterPort,
    /// Two qualified delegations on one port match the same carried node.
    AmbiguousDelegation,
    /// Attempt to delete or divergently redeclare a stdlib element.
    StdlibProtected,
    /// A connection crosses a scope boundary, or an application's inner node is
    /// not a direct child of the delegating node.
    CrossScope,
}

impl ErrorCode {
    /// The stable string identifier of this code.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::Parse => "E_PARSE",
            ErrorCode::UnknownName => "E_UNKNOWN_NAME",
            ErrorCode::DupName => "E_DUP_NAME",
            ErrorCode::Redeclared => "E_REDECLARED",
            ErrorCode::ShapeViolation => "E_SHAPE_VIOLATION",
            ErrorCode::CarrierRequired => "E_CARRIER_REQUIRED",
            ErrorCode::CarrierForbidden => "E_CARRIER_FORBIDDEN",
            ErrorCode::PortTypeConflict => "E_PORT_TYPE_CONFLICT",
            ErrorCode::PortSideConflict => "E_PORT_SIDE_CONFLICT",
            ErrorCode::NoOuterPort => "E_NO_OUTER_PORT",
            ErrorCode::AmbiguousDelegation => "E_AMBIGUOUS_DELEGATION",
            ErrorCode::StdlibProtected => "E_STDLIB_PROTECTED",
            ErrorCode::CrossScope => "E_CROSS_SCOPE",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reference to an element involved in an error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorRef {
    /// What the reference points at: `"node"`, `"rel"`, `"conn"`, `"view"`,
    /// `"port"`, `"edge"`, `"slot"` or `"path"`.
    pub kind: &'static str,
    /// Rendered path or name of the element, as addressable from the root scope.
    pub path: String,
    /// The element's stable id, when it exists in the store.
    pub id: Option<u64>,
}

/// A rejected statement. The model is untouched.
#[derive(Clone, Debug)]
pub struct LangError {
    /// Stable identifier from the catalog.
    pub code: ErrorCode,
    /// Human-readable one-liner.
    pub message: String,
    /// The offending statement, as submitted.
    pub subject: String,
    /// Paths/ids of the elements involved.
    pub refs: Vec<ErrorRef>,
    /// The violated constraint, where applicable: what was required.
    pub expected: Option<String>,
    /// The violated constraint, where applicable: what was found.
    pub actual: Option<String>,
    /// Suggested next step, phrased as a runnable statement or query.
    pub hint: Option<String>,
}

impl LangError {
    pub(crate) fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        LangError {
            code,
            message: message.into(),
            subject: String::new(),
            refs: Vec::new(),
            expected: None,
            actual: None,
            hint: None,
        }
    }

    pub(crate) fn with_ref(
        mut self,
        kind: &'static str,
        path: impl Into<String>,
        id: Option<u64>,
    ) -> Self {
        self.refs.push(ErrorRef {
            kind,
            path: path.into(),
            id,
        });
        self
    }

    pub(crate) fn with_expected(mut self, e: impl Into<String>) -> Self {
        self.expected = Some(e.into());
        self
    }

    pub(crate) fn with_actual(mut self, a: impl Into<String>) -> Self {
        self.actual = Some(a.into());
        self
    }

    pub(crate) fn with_hint(mut self, h: impl Into<String>) -> Self {
        self.hint = Some(h.into());
        self
    }
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if !self.subject.is_empty() {
            write!(f, " (subject: {})", self.subject)?;
        }
        if let Some(e) = &self.expected {
            write!(f, " expected: {e};")?;
        }
        if let Some(a) = &self.actual {
            write!(f, " actual: {a};")?;
        }
        if let Some(h) = &self.hint {
            write!(f, " hint: {h}")?;
        }
        Ok(())
    }
}

impl std::error::Error for LangError {}
