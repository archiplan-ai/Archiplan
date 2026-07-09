//! Statement results, findings, and the request/response envelope of the
//! agent interface (`requirements/agent-interface.md`).

use std::fmt;

use serde::Serialize;
use serde_json::Value;

use crate::error::LangError;
use crate::statement::{EdgeKind, PatternExpr, Statement};

/// The result of one statement, tagged by outcome.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum Outcome {
    /// The model changed.
    Applied,
    /// The statement restates something identical that already exists.
    Noop,
    /// `query` output: the sliced subgraph as plain nodes and edges.
    Graph {
        /// The nodes of the slice, in creation order.
        nodes: Vec<GraphNode>,
        /// The edges of the slice, in creation order.
        edges: Vec<GraphEdge>,
    },
    /// `check` output: model-completeness findings.
    Findings {
        /// The findings.
        findings: Vec<Finding>,
    },
}

/// One node of a query result. The result is a common node-link graph: edges
/// reference nodes by `id`, which is the node's absolute path.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GraphNode {
    /// The node's absolute path — the stable string id edges reference.
    pub id: String,
    /// The node's own name (the last path segment).
    pub name: String,
    /// The node's definition — its identity prose; omitted when it has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Absolute paths of the nodes classifying this one via `type_of`,
    /// following the transitive closure; omitted when unclassified.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    /// The node's ports referenced by edges of this result; omitted when none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<GraphPort>,
}

/// A port as it appears in query results.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GraphPort {
    /// The port name, unique on its node.
    pub name: String,
    /// The connection type the port is fixed to.
    pub conn: String,
    /// `source` or `target` for ports of directed connection types; omitted
    /// for undirected ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<&'static str>,
}

/// One edge of a query result. `source`/`target` are node ids (absolute
/// paths); kind-specific fields are omitted where they do not apply.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GraphEdge {
    /// The edge kind: `relation`, `connection` or `application`.
    pub kind: EdgeKind,
    /// The rel/conn type name; applications are untyped.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    /// Whether the type is directed; omitted on applications, whose
    /// orientation is the outer→inner mapping itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directed: Option<bool>,
    /// Source node id; the delegating (outer) node on applications.
    pub source: String,
    /// The port the edge attaches to on the source node; relations attach to
    /// the node as a whole.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<String>,
    /// Target node id; the inner node on applications.
    pub target: String,
    /// The port the edge attaches to on the target node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_port: Option<String>,
    /// The forward-lane carried node's id. Metadata: the carrier is not an
    /// attachment and need not be a node of the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier: Option<String>,
    /// The reverse-lane carried node's id (request/response connections).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev_carrier: Option<String>,
    /// The carried-node qualifier of a delegation, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<PatternExpr>,
    /// Names of the views the edge belongs to; for applications, the views of
    /// the connection edges they route. Omitted when untagged.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub views: Vec<String>,
}

/// A batch failed: the whole batch was rolled back.
#[derive(Clone, Debug)]
pub struct BatchError {
    /// Index of the failing statement within the batch.
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

/// A model-completeness finding: a state that is legal mid-construction but
/// suspect. Findings are surfaced by `check`, never by rejecting writes.
/// Kinds are append-only, like error codes.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Finding {
    /// Carried traffic on a delegated port that matches no delegation.
    UnroutedTraffic {
        /// The unroutable connection edge, as a statement.
        statement: Box<Statement>,
        /// The delegated port, as `path.port`.
        port: String,
    },
    /// A declared port with no attached connections or applications.
    UnusedPort {
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
        type_kind: &'static str,
        /// The type name.
        name: String,
    },
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::UnroutedTraffic { statement, port } => {
                write!(
                    f,
                    "unrouted traffic: {} — no delegation on {port} matches",
                    statement.pseudo()
                )
            }
            Finding::UnusedPort { port } => {
                write!(f, "declared port without attachments: {port}")
            }
            Finding::EmptyView { view } => write!(f, "empty view: {view}"),
            Finding::TypeWithoutInstances { type_kind, name } => {
                write!(f, "type without instances: {type_kind} {name}")
            }
        }
    }
}

/// A parsed request envelope: one batch of read statements against one model.
#[derive(Clone, Debug)]
pub struct Request {
    /// The batch: raw statement values, parsed per statement so a schema
    /// error reports the failing index.
    pub statements: Vec<Value>,
}

/// The response envelope.
#[derive(Clone, Debug, Serialize)]
pub struct Response {
    /// `ok` or `error`.
    pub status: &'static str,
    /// One entry per statement, in batch order (when `status` is `ok`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<Outcome>>,
    /// The failure (when `status` is `error`); the whole batch was rolled back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// The error member of a response: the statement error plus the failing
/// statement's index; protocol errors carry no index.
#[derive(Clone, Debug, Serialize)]
pub struct ResponseError {
    /// Index of the failing statement, absent for protocol errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    /// The error itself, flattened into this object.
    #[serde(flatten)]
    pub error: LangError,
}

impl Response {
    pub(crate) fn ok(results: Vec<Outcome>) -> Self {
        Response {
            status: "ok",
            results: Some(results),
            error: None,
        }
    }

    pub(crate) fn fail(index: Option<usize>, error: LangError) -> Self {
        Response {
            status: "error",
            results: None,
            error: Some(ResponseError { index, error }),
        }
    }
}
