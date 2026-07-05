//! The error contract: structured, machine-readable statement failures.
//!
//! Every rejected statement produces a [`LangError`] with a stable [`ErrorCode`],
//! the offending statement as submitted, the elements involved, the violated
//! constraint where applicable, and a suggested next step phrased as a runnable
//! statement. Codes are append-only.

use std::fmt;

use serde::Serialize;
use serde_json::Value;

/// Stable error codes from the catalog in `requirements/modeling-lang/errors.md`,
/// plus the protocol-level codes of `requirements/agent-interface.md`
/// (`BadRequest`, `StaleRevision`) which concern the envelope rather than a
/// statement.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorCode {
    /// The statement is not a well-formed statement object: unknown `stmt`,
    /// missing or ill-typed field, malformed path.
    Parse,
    /// A referenced node / type / view / path does not resolve — including
    /// `redefine` of an element that does not exist.
    UnknownName,
    /// A rename collides with a sibling's name.
    DupName,
    /// A `define` differs from the existing definition of the name —
    /// including a rel / conn kind mismatch.
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
    /// A connection joins nodes of different scopes.
    CrossScope,
    /// Attempt to delete or divergently redefine a stdlib element.
    StdlibProtected,
    /// The request envelope is not valid or violates the contract.
    BadRequest,
    /// `expect_revision` does not match the model's current revision.
    StaleRevision,
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
            ErrorCode::CrossScope => "E_CROSS_SCOPE",
            ErrorCode::StdlibProtected => "E_STDLIB_PROTECTED",
            ErrorCode::BadRequest => "E_BAD_REQUEST",
            ErrorCode::StaleRevision => "E_STALE_REVISION",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// A reference to an element involved in an error.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ErrorRef {
    /// What the reference points at: `"node"`, `"rel"`, `"conn"`, `"view"`,
    /// `"port"`, `"edge"` or `"slot"`.
    pub kind: &'static str,
    /// Rendered absolute path or name of the element.
    pub path: String,
    /// The element's stable id, when it exists in the store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
}

/// A rejected statement or request. The model is untouched.
#[derive(Clone, Debug, Serialize)]
pub struct LangError {
    /// Stable identifier from the catalog.
    pub code: ErrorCode,
    /// Human-readable one-liner.
    pub message: String,
    /// The offending statement, as submitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Value>,
    /// Paths/ids of the elements involved.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<ErrorRef>,
    /// The violated constraint, where applicable: what was required
    /// (a pattern object, a type name, ...).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<Value>,
    /// The violated constraint, where applicable: what was found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<Value>,
    /// Suggested next step, phrased as a runnable statement object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<Value>,
}

impl LangError {
    pub(crate) fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        LangError {
            code,
            message: message.into(),
            subject: None,
            refs: Vec::new(),
            expected: None,
            actual: None,
            hint: None,
        }
    }

    pub(crate) fn with_subject(mut self, subject: Value) -> Self {
        self.subject = Some(subject);
        self
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

    pub(crate) fn with_expected(mut self, e: Value) -> Self {
        self.expected = Some(e);
        self
    }

    pub(crate) fn with_actual(mut self, a: Value) -> Self {
        self.actual = Some(a);
        self
    }

    pub(crate) fn with_hint(mut self, h: Value) -> Self {
        self.hint = Some(h);
        self
    }
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(e) = &self.expected {
            write!(f, "; expected {e}")?;
        }
        if let Some(a) = &self.actual {
            write!(f, "; actual {a}")?;
        }
        Ok(())
    }
}

impl std::error::Error for LangError {}
