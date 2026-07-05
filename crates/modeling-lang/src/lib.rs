//! The Archiplan modeling language: a substrate to express arbitrary
//! architectures in a structured way that can be transformed or queried later.
//!
//! Implements `requirements/modeling-lang/` — the free graph of nodes (with
//! ports and nested scopes) and distinguished edges (relations, connections,
//! applications), views, the JSON statement API with its idempotent
//! definitions and error contract, cascading deletion, and the read
//! statements (`ports`, `check`, `dump`) — plus the request/response envelope
//! of `requirements/agent-interface.md`.
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
//! - A statement is a JSON object discriminated by `stmt`; JSON is the only
//!   parsed syntax. The compact pseudo-syntax (`def node Payments;`) is
//!   render-only: [`Statement::pseudo`] produces it for human output.
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
//! - The stdlib `type_of` is excluded from dumps and from the "type without
//!   instances" finding: it is substrate, not model intent.
//! - The revision increments once per model-changing request.
//! - Names are `[A-Za-z_][A-Za-z0-9_]*`; paths join them with `.`.

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
mod query;
mod render;
mod result;
mod statement;

pub use engine::Workspace;
pub use error::{ErrorCode, ErrorRef, LangError};
pub use model::{Layer, Model};
pub use result::{BatchError, Finding, Outcome, Request, Response, ResponseError};
pub use statement::{Definition, End, PatternExpr, Statement, parse_statement};
