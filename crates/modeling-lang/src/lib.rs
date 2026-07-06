//! The Archiplan modeling language: a substrate to express arbitrary
//! architectures in a structured way that can be transformed or queried later.
//!
//! Implements `requirements/modeling-lang/` — the free graph of nodes (with
//! ports and nested scopes) and distinguished edges (relations, connections,
//! applications), views, the JSON statement API with its idempotent
//! definitions and error contract, cascading deletion, and the read
//! statements (the subgraph `query` and `check`) — plus the `.arch` source
//! format (`requirements/modeling-lang/source-format.md`), the
//! request/response envelope of `requirements/agent-interface.md`, ontology
//! presets loaded as the standard library
//! (`requirements/modeling-lang/ontology.md`), and NKP landscape analysis
//! (`requirements/scoring/nkp.md`).
//!
//! # Quick start
//!
//! ```
//! use modeling_lang::Workspace;
//! use serde_json::json;
//!
//! let mut ws = Workspace::new();
//! let response = ws.handle(&json!({
//!     "statements": [
//!         { "stmt": "define", "node": "Service" },
//!         { "stmt": "define", "node": "Payments" },
//!         { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
//!         { "stmt": "check" }
//!     ]
//! }));
//! assert_eq!(response.status, "ok");
//! assert_eq!(response.revision, 1);
//! ```
//!
//! # Semantics
//!
//! - A statement is a JSON object discriminated by `stmt`. The `.arch`
//!   source format ([`source`]) is the human syntax: a project of text files
//!   compiled down to the same statements ([`source::compile_project`]).
//!   [`Statement::pseudo`] renders statements back in it, so dumps and
//!   cascades are pasteable source.
//! - There is no ambient scope: every reference is an absolute path, creation
//!   statements carry the full path of what they create, applications name
//!   their delegating node explicitly. Augmentation is just a statement whose
//!   path lands inside an existing node.
//! - Named elements are created by `define` and replaced by `redefine`; the
//!   [`Definition`] names its subject (`node`, `view`, `rel`, `conn`) and
//!   parameters. Both are idempotent: `define` creates or no-ops on an
//!   identical restatement (a divergent one is rejected); `redefine` requires
//!   existence and no-ops when nothing changes (a node redefine empties its
//!   scope as a reported cascade, a type redefine replaces the shape and lets
//!   nonconforming edges drift into findings).
//! - Batches are atomic; every statement applies, no-ops, or fails with a
//!   structured [`LangError`], and a failure rolls the whole batch back.
//! - `delete` cascades over the full referencing closure and reports
//!   everything removed as replayable statements. Shape conformance is soft:
//!   edits that erode it succeed and surface later as [`Finding`]s via `check`.
//!
//! # Implementation decisions
//!
//! Where the requirement leaves room, this implementation chooses:
//!
//! - Pattern matching (`(Service type_of *)`) follows the virtual transitive
//!   closure of a `trans` relation; only declared edges are stored.
//! - A node `redefine` whose scope is already empty is a no-op.
//! - The stdlib is a [`Preset`]: a creation-only statement batch loaded
//!   before user statements ([`Workspace::with_preset`]). [`Workspace::new`]
//!   loads [`Preset::core`] — exactly the historical stdlib, `type_of` only.
//!   Every preset must define `type_of` conforming to
//!   `rel trans type_of := * -> *`. Preset elements are excluded from dumps
//!   and findings and are protected from mutation (`E_STDLIB_PROTECTED`);
//!   users may reference them, attach edges to them, and augment their
//!   scopes.
//! - The revision increments once per model-changing request.
//! - Names are `[A-Za-z_][A-Za-z0-9_]*`; paths join them with `.`.
//!
//! For the subgraph `query` (`requirements/modeling-lang/queries.md`):
//!
//! - Each filter is optional and absent means unrestricted; present filters
//!   compose by intersection. An empty list is the most restrictive filter of
//!   its category (`"scopes": []` is "the top level only"), not an absent one.
//! - `types` keeps the *instances* of the listed types — nodes classified via
//!   the transitive `type_of` closure; the type node itself does not match.
//! - `scopes` names the scopes to open: each entry opens the chain from the
//!   root down to it plus its whole subtree; the top level is always open. A
//!   node is included only when every scope containing it is open.
//! - `views` keeps the edges of the listed views and only the nodes related
//!   to them — their attachments and carried nodes.
//! - An edge survives only if all its attachments survive the node filters;
//!   the carrier is edge metadata and neither anchors nor blocks inclusion.
//! - Results are a common node-link JSON graph: node ids are absolute paths,
//!   nodes and edges come in creation order, node `types` list all transitive
//!   classifiers, node `ports` list the ports referenced by result edges.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
// LangError carries the full structured contract (subject, refs, expected /
// actual, hint) and flows through interpreter statement paths where error-path
// size is irrelevant; boxing it would push the cost onto every consumer match.
#![allow(clippy::result_large_err)]

mod cascade;
mod engine;
mod error;
mod ids;
mod model;
mod nkp;
mod preset;
mod query;
mod render;
mod result;
pub mod source;
mod statement;

pub use engine::Workspace;
pub use error::{ErrorCode, ErrorRef, LangError};
pub use model::{Layer, Model};
pub use nkp::{
    CorridorAction, CorridorLabel, ExcludePattern, Hotspot, Neutrality, NkpConfig, NkpCorridor,
    NkpMatrix, NkpMetrics, NkpReport, NkpScope, NkpScopeInfo, NkpWarning, Regime, Slot,
};
pub use preset::Preset;
pub use result::{
    BatchError, Finding, GraphEdge, GraphNode, GraphPort, Outcome, Request, Response, ResponseError,
};
pub use statement::{Definition, EdgeKind, End, PatternExpr, Statement, parse_statement};
