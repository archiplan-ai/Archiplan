//! The Archiplan modeling language: a substrate to express arbitrary
//! architectures in a structured way that can be transformed or queried later.
//!
//! Implements `requirements/modeling-lang/` — the free graph of nodes (with
//! ports and nested scopes) and distinguished edges (relations, connections,
//! applications), views, the statement API with its error contract, cascading
//! deletion, and the query operations (`ports`, `check`, `dump`).
//!
//! # Quick start
//!
//! ```
//! use modeling_lang::Session;
//!
//! let mut session = Session::new();
//! session
//!     .execute(
//!         r#"
//!         node Service;
//!         node Payments;
//!         node Orders;
//!         Service type_of Payments;
//!         Service type_of Orders;
//!
//!         node OrderId;
//!         conn confirm := (Service type_of *) (OrderId)-> (Service type_of *);
//!         Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);
//!
//!         node Orders {
//!             node ConfirmationHandler;
//!             handle_confirmation = ConfirmationHandler(handle_confirmation);
//!         }
//!         "#,
//!     )
//!     .expect("batch applies");
//!
//! let results = session.execute("ports Orders").expect("query runs");
//! ```
//!
//! # Semantics
//!
//! - Every element has an opaque, immutable **id** and a scoped **name**;
//!   everything stored (edge ends, pattern anchors, delegations) binds to ids,
//!   so `rename` is reference-safe by construction. Edges carry no name: an
//!   edge is addressed structurally, by restating it.
//! - Statements are atomic. [`Session::execute`] applies a batch atomically;
//!   [`Session::execute_interactive`] applies statements one at a time. Every
//!   statement applies, no-ops (identical restatement), or fails with a
//!   structured [`LangError`] that leaves the model untouched.
//! - `delete` cascades over the full referencing closure and reports
//!   everything removed, rendered as statements. Shape conformance is soft:
//!   edits that erode it succeed and surface later as [`Finding`]s via `check`.
//!
//! # Implementation decisions
//!
//! Where the requirement leaves room, this implementation chooses:
//!
//! - **`E_CROSS_SCOPE`** is appended to the error catalog (the catalog is
//!   append-only): connections must join nodes of the same scope — crossing a
//!   boundary is what applications are for — and an application's inner node
//!   must be a direct child of the delegating node. Relations are free to
//!   relate nodes anywhere: the epistemic layer classifies terms in any scope.
//! - Pattern matching (`(Service type_of *)`) follows the virtual transitive
//!   closure of a `trans` relation. Only declared edges are stored; the
//!   closure is computed on demand.
//! - A `dump [in views]` query is provided as the whole-model (or view-sliced)
//!   render; results replay from the root scope in creation order.
//! - The stdlib `type_of` is excluded from the "type without instances"
//!   finding: it is substrate, not model intent.
//! - Reserved words (`node`, `view`, `rel`, `conn`, `trans`, `open`, `rename`,
//!   `delete`, `untag`, `in`, `ports`, `check`, `dump`) cannot be used as
//!   names.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
// LangError carries the full structured contract (subject, refs, expected /
// actual, hint) and flows through interpreter statement paths where error-path
// size is irrelevant; boxing it would push the cost onto every consumer match.
#![allow(clippy::result_large_err)]

mod ast;
mod cascade;
mod error;
mod ids;
mod lexer;
mod model;
mod parser;
mod query;
mod render;
mod result;
mod session;

pub use error::{ErrorCode, ErrorRef, LangError};
pub use model::{Layer, Model};
pub use result::{BatchError, Finding, InteractiveResult, Outcome, StatementResult};
pub use session::Session;
