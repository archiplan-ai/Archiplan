//! The statement engine: applies batches of statements to a model.
//!
//! A batch is atomic: every statement either applies, is an
//! identical-restatement no-op, or fails with a structured error that rolls
//! the whole batch back. There is no session state — every statement is
//! absolutely addressed and self-contained.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::cascade::{self, Seed};
use crate::error::{ErrorCode, LangError};
use crate::ids::{ConnId, EdgeId, NodeId, PortId, RelId, ViewId};
use crate::model::{
    ConnType, Edge, EdgePayload, Model, Node, Pattern, Port, RelType, Side, ViewDef,
};
use crate::query;
use crate::result::{BatchError, Outcome, Request, Response};
use crate::statement::{Definition, End, PatternExpr, Statement, parse_statement};

/// A model plus the revision counter maintained for the agent interface.
///
/// [`Workspace::execute`] runs a parsed batch; [`Workspace::handle`] speaks
/// the request/response envelope of `requirements/agent-interface.md`.
#[derive(Clone, Debug)]
pub struct Workspace {
    model: Model,
    revision: u64,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Workspace {
    /// A fresh workspace on an empty model with the standard library loaded.
    pub fn new() -> Self {
        Workspace {
            model: Model::new_with_stdlib(),
            revision: 0,
        }
    }

    /// Rebuild a workspace by replaying `statements` (typically a dump) and
    /// adopting the given revision.
    pub fn restore(revision: u64, statements: &[Statement]) -> Result<Self, BatchError> {
        let mut ws = Workspace::new();
        ws.execute(statements)?;
        ws.revision = revision;
        Ok(ws)
    }

    /// Read access to the model.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// The current revision: increases whenever the model changes, untouched
    /// by noops, reads and dry runs.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Execute a parsed batch atomically. On success returns one outcome per
    /// statement; any error rolls the whole batch back.
    pub fn execute(&mut self, statements: &[Statement]) -> Result<Vec<Outcome>, BatchError> {
        let checkpoint = self.model.clone();
        let mut results = Vec::new();
        for (index, stmt) in statements.iter().enumerate() {
            match self.apply(stmt) {
                Ok(outcome) => results.push(outcome),
                Err(mut error) => {
                    self.model = checkpoint;
                    if error.subject.is_none() {
                        error.subject = Some(stmt.to_value());
                    }
                    return Err(BatchError { index, error });
                }
            }
        }
        if results.iter().any(Outcome::changed_model) {
            self.revision += 1;
        }
        Ok(results)
    }

    /// Parse raw statement values and execute them as one atomic batch; a
    /// schema error reports the failing statement's index.
    pub fn execute_values(&mut self, values: &[Value]) -> Result<Vec<Outcome>, BatchError> {
        let mut statements = Vec::with_capacity(values.len());
        for (index, v) in values.iter().enumerate() {
            match parse_statement(v) {
                Ok(s) => statements.push(s),
                Err(error) => return Err(BatchError { index, error }),
            }
        }
        self.execute(&statements)
    }

    /// Handle one request envelope: `{ "statements": [...], "expect_revision"?,
    /// "dry_run"? }`. Envelope violations are protocol errors (`E_BAD_REQUEST`,
    /// `E_STALE_REVISION`); statement failures carry the failing index.
    pub fn handle(&mut self, request: &Value) -> Response {
        let req = match parse_request(request) {
            Ok(r) => r,
            Err(e) => return Response::fail(self.revision, None, e),
        };
        if let Some(expected) = req.expect_revision
            && expected != self.revision
        {
            let e = LangError::new(
                ErrorCode::StaleRevision,
                format!("the model is at revision {}, not {expected}", self.revision),
            )
            .with_expected(json!(expected))
            .with_actual(json!(self.revision))
            .with_hint(json!({ "stmt": "query" }));
            return Response::fail(self.revision, None, e);
        }
        if req.dry_run {
            let mut probe = self.clone();
            match probe.execute_values(&req.statements) {
                Ok(results) => Response::ok(self.revision, results),
                Err(b) => Response::fail(self.revision, Some(b.index), b.error),
            }
        } else {
            match self.execute_values(&req.statements) {
                Ok(results) => Response::ok(self.revision, results),
                Err(b) => Response::fail(self.revision, Some(b.index), b.error),
            }
        }
    }

    // ---- statement dispatch ------------------------------------------------

    fn apply(&mut self, stmt: &Statement) -> Result<Outcome, LangError> {
        match stmt {
            Statement::Define(def) => self.do_define(def),
            Statement::Redefine(def) => self.do_redefine(def),
            Statement::RelEdge {
                rel,
                source,
                target,
                views,
            } => self.do_rel_edge(rel, source, target, views),
            Statement::ConnEdge {
                conn,
                source,
                carrier,
                target,
                views,
            } => self.do_conn_edge(conn, source, carrier.as_deref(), target, views),
            Statement::App {
                node,
                port,
                route,
                inner,
            } => self.do_app(node, port, route.as_ref(), inner),
            Statement::Rename { node, to } => self.do_rename(node, to),
            Statement::Delete {
                node,
                edge,
                rel,
                conn,
                view,
            } => self.do_delete(
                node.as_deref(),
                edge.as_deref(),
                rel.as_deref(),
                conn.as_deref(),
                view.as_deref(),
            ),
            Statement::Untag { edge, views } => self.do_untag(edge, views),
            Statement::Query {
                types,
                kinds,
                views,
                scopes,
            } => {
                let resolve_all = |paths: &[String]| -> Result<Vec<NodeId>, LangError> {
                    paths.iter().map(|p| self.resolve_abs(p)).collect()
                };
                let filter = query::SubgraphFilter {
                    types: types.as_deref().map(resolve_all).transpose()?,
                    kinds: kinds.as_ref().map(|ks| ks.iter().copied().collect()),
                    views: views
                        .as_deref()
                        .map(|vs| self.resolve_views(vs))
                        .transpose()?,
                    scopes: scopes.as_deref().map(resolve_all).transpose()?,
                };
                let (nodes, edges) = query::subgraph(&self.model, &filter);
                Ok(Outcome::Graph { nodes, edges })
            }
            Statement::Check { in_views } => {
                let filter = self.view_filter(in_views)?;
                Ok(Outcome::Findings {
                    findings: query::check(&self.model, filter.as_ref()),
                })
            }
        }
    }

    // ---- resolution ---------------------------------------------------------

    fn parse_path(&self, path: &str) -> Result<Vec<String>, LangError> {
        let segs: Vec<String> = path.split('.').map(str::to_string).collect();
        if segs.iter().all(|s| is_ident(s)) {
            Ok(segs)
        } else {
            Err(LangError::new(
                ErrorCode::Parse,
                format!("malformed path `{path}`"),
            ))
        }
    }

    fn check_ident(&self, name: &str, what: &str) -> Result<(), LangError> {
        if is_ident(name) {
            Ok(())
        } else {
            Err(LangError::new(
                ErrorCode::Parse,
                format!("malformed {what} `{name}`"),
            ))
        }
    }

    fn unknown(&self, kind: &'static str, what: &str) -> LangError {
        LangError::new(ErrorCode::UnknownName, format!("unknown {kind} `{what}`"))
            .with_ref(kind, what, None)
            .with_hint(json!({ "stmt": "query" }))
    }

    fn resolve_abs(&self, path: &str) -> Result<NodeId, LangError> {
        let segs = self.parse_path(path)?;
        self.model
            .resolve_in(None, &segs)
            .ok_or_else(|| self.unknown("node", path))
    }

    fn resolve_pattern(&self, expr: &PatternExpr) -> Result<Pattern, LangError> {
        Ok(match expr {
            PatternExpr::Any => Pattern::Any,
            PatternExpr::Exact { node } => Pattern::Exact(self.resolve_abs(node)?),
            PatternExpr::Classified { anchor, rel } => {
                let a = self.resolve_abs(anchor)?;
                let r = *self
                    .model
                    .rel_names
                    .get(rel)
                    .ok_or_else(|| self.unknown("rel", rel))?;
                Pattern::Classified { anchor: a, rel: r }
            }
        })
    }

    fn resolve_views(&self, names: &[String]) -> Result<BTreeSet<ViewId>, LangError> {
        names
            .iter()
            .map(|n| {
                self.model.view_names.get(n).copied().ok_or_else(|| {
                    self.unknown("view", n)
                        .with_hint(json!({ "stmt": "define", "view": n }))
                })
            })
            .collect()
    }

    fn view_filter(&self, names: &[String]) -> Result<Option<BTreeSet<ViewId>>, LangError> {
        if names.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.resolve_views(names)?))
        }
    }

    // ---- definitions -----------------------------------------------------------

    fn do_define(&mut self, def: &Definition) -> Result<Outcome, LangError> {
        match def {
            Definition::Node { path } => self.define_node(path),
            Definition::View { name } => self.define_view(name),
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
            } => self.define_rel(name, *trans, *directed, source, target),
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                target,
            } => self.define_conn(name, *directed, source, carrier.as_ref(), target),
        }
    }

    fn do_redefine(&mut self, def: &Definition) -> Result<Outcome, LangError> {
        match def {
            Definition::Node { path } => self.redefine_node(path),
            // Rejected at parse time.
            Definition::View { .. } => Err(LangError::new(
                ErrorCode::Parse,
                "a view has no definition body; `redefine` does not apply (`define` only)",
            )),
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
            } => self.redefine_rel(name, *trans, *directed, source, target),
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                target,
            } => self.redefine_conn(name, *directed, source, carrier.as_ref(), target),
        }
    }

    fn define_node(&mut self, path: &str) -> Result<Outcome, LangError> {
        let segs = self.parse_path(path)?;
        let (name, prefix) = segs.split_last().expect("paths are non-empty");
        let parent = if prefix.is_empty() {
            None
        } else {
            Some(self.model.resolve_in(None, prefix).ok_or_else(|| {
                self.unknown("node", &prefix.join("."))
                    .with_ref("path", path, None)
            })?)
        };
        if self.model.children(parent).contains_key(name) {
            return Ok(Outcome::Noop);
        }
        let id = NodeId(self.model.alloc());
        self.model.nodes.insert(
            id,
            Node {
                id,
                name: name.clone(),
                parent,
                children: BTreeMap::new(),
                ports: BTreeMap::new(),
            },
        );
        match parent {
            Some(p) => {
                self.model
                    .nodes
                    .get_mut(&p)
                    .expect("parent exists")
                    .children
                    .insert(name.clone(), id);
            }
            None => {
                self.model.root.insert(name.clone(), id);
            }
        }
        Ok(Outcome::applied())
    }

    /// Redefinition keeps the node's identity, ports and attached edges and
    /// empties its scope as a reported cascade; an already-empty scope is a
    /// no-op.
    fn redefine_node(&mut self, path: &str) -> Result<Outcome, LangError> {
        let id = self
            .resolve_abs(path)
            .map_err(|e| e.with_hint(json!({ "stmt": "define", "node": path })))?;
        let children: Vec<Seed> = self.model.nodes[&id]
            .children
            .values()
            .map(|c| Seed::Node(*c))
            .collect();
        if children.is_empty() {
            return Ok(Outcome::Noop);
        }
        let cascade = cascade::delete_many(&mut self.model, children);
        Ok(Outcome::Applied {
            cascade: Some(cascade),
        })
    }

    fn define_view(&mut self, name: &str) -> Result<Outcome, LangError> {
        self.check_ident(name, "view name")?;
        if self.model.view_names.contains_key(name) {
            return Ok(Outcome::Noop);
        }
        let id = ViewId(self.model.alloc());
        self.model.views.insert(
            id,
            ViewDef {
                id,
                name: name.to_string(),
            },
        );
        self.model.view_names.insert(name.to_string(), id);
        Ok(Outcome::applied())
    }

    fn define_rel(
        &mut self,
        name: &str,
        trans: bool,
        directed: bool,
        source: &PatternExpr,
        target: &PatternExpr,
    ) -> Result<Outcome, LangError> {
        self.check_ident(name, "relation type name")?;
        let src = self.resolve_pattern(source)?;
        let dst = self.resolve_pattern(target)?;
        if let Some(&id) = self.model.rel_names.get(name) {
            let existing = &self.model.rels[&id];
            let identical = existing.trans == trans
                && existing.directed == directed
                && existing.src == src
                && existing.dst == dst;
            if identical {
                return Ok(Outcome::Noop);
            }
            let mut e = LangError::new(
                ErrorCode::Redeclared,
                format!("rel `{name}` is already defined differently"),
            )
            .with_ref("rel", name, Some(id.raw()))
            .with_actual(self.model.rel_statement(existing).to_value());
            if !existing.stdlib {
                let hint = Statement::Redefine(rel_def(name, trans, directed, source, target));
                e = e.with_hint(hint.to_value());
            }
            return Err(e);
        }
        if let Some(&cid) = self.model.conn_names.get(name) {
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("`{name}` is defined as a connection type, not a relation"),
            )
            .with_ref("conn", name, Some(cid.raw()))
            .with_actual(
                self.model
                    .conn_statement(&self.model.conns[&cid])
                    .to_value(),
            ));
        }
        let id = RelId(self.model.alloc());
        self.model.rels.insert(
            id,
            RelType {
                id,
                name: name.to_string(),
                trans,
                directed,
                src,
                dst,
                stdlib: false,
            },
        );
        self.model.rel_names.insert(name.to_string(), id);
        Ok(Outcome::applied())
    }

    fn redefine_rel(
        &mut self,
        name: &str,
        trans: bool,
        directed: bool,
        source: &PatternExpr,
        target: &PatternExpr,
    ) -> Result<Outcome, LangError> {
        self.check_ident(name, "relation type name")?;
        let src = self.resolve_pattern(source)?;
        let dst = self.resolve_pattern(target)?;
        if let Some(&id) = self.model.rel_names.get(name) {
            let existing = &self.model.rels[&id];
            let identical = existing.trans == trans
                && existing.directed == directed
                && existing.src == src
                && existing.dst == dst;
            if identical {
                return Ok(Outcome::Noop);
            }
            if existing.stdlib {
                return Err(LangError::new(
                    ErrorCode::StdlibProtected,
                    format!("`{name}` is a stdlib relation and cannot be redefined divergently"),
                )
                .with_actual(self.model.rel_statement(existing).to_value()));
            }
            let rt = self.model.rels.get_mut(&id).expect("rel exists");
            rt.trans = trans;
            rt.directed = directed;
            rt.src = src;
            rt.dst = dst;
            return Ok(Outcome::applied());
        }
        if let Some(&cid) = self.model.conn_names.get(name) {
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("`{name}` is defined as a connection type, not a relation"),
            )
            .with_ref("conn", name, Some(cid.raw()))
            .with_actual(
                self.model
                    .conn_statement(&self.model.conns[&cid])
                    .to_value(),
            ));
        }
        Err(self.unknown("rel", name).with_hint(
            Statement::Define(rel_def(name, trans, directed, source, target)).to_value(),
        ))
    }

    fn define_conn(
        &mut self,
        name: &str,
        directed: bool,
        source: &PatternExpr,
        carrier: Option<&PatternExpr>,
        target: &PatternExpr,
    ) -> Result<Outcome, LangError> {
        self.check_ident(name, "connection type name")?;
        let src = self.resolve_pattern(source)?;
        let carrier_pat = carrier.map(|c| self.resolve_pattern(c)).transpose()?;
        let dst = self.resolve_pattern(target)?;
        if let Some(&id) = self.model.conn_names.get(name) {
            let existing = &self.model.conns[&id];
            let identical = existing.directed == directed
                && existing.src == src
                && existing.carrier == carrier_pat
                && existing.dst == dst;
            if identical {
                return Ok(Outcome::Noop);
            }
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("conn `{name}` is already defined differently"),
            )
            .with_ref("conn", name, Some(id.raw()))
            .with_actual(self.model.conn_statement(existing).to_value())
            .with_hint(
                Statement::Redefine(conn_def(name, directed, source, carrier, target)).to_value(),
            ));
        }
        if let Some(&rid) = self.model.rel_names.get(name) {
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("`{name}` is defined as a relation type, not a connection"),
            )
            .with_ref("rel", name, Some(rid.raw()))
            .with_actual(self.model.rel_statement(&self.model.rels[&rid]).to_value()));
        }
        let id = ConnId(self.model.alloc());
        self.model.conns.insert(
            id,
            ConnType {
                id,
                name: name.to_string(),
                directed,
                src,
                carrier: carrier_pat,
                dst,
            },
        );
        self.model.conn_names.insert(name.to_string(), id);
        Ok(Outcome::applied())
    }

    fn redefine_conn(
        &mut self,
        name: &str,
        directed: bool,
        source: &PatternExpr,
        carrier: Option<&PatternExpr>,
        target: &PatternExpr,
    ) -> Result<Outcome, LangError> {
        self.check_ident(name, "connection type name")?;
        let src = self.resolve_pattern(source)?;
        let carrier_pat = carrier.map(|c| self.resolve_pattern(c)).transpose()?;
        let dst = self.resolve_pattern(target)?;
        if let Some(&id) = self.model.conn_names.get(name) {
            let existing = &self.model.conns[&id];
            let identical = existing.directed == directed
                && existing.src == src
                && existing.carrier == carrier_pat
                && existing.dst == dst;
            if identical {
                return Ok(Outcome::Noop);
            }
            let ct = self.model.conns.get_mut(&id).expect("conn exists");
            ct.directed = directed;
            ct.src = src;
            ct.carrier = carrier_pat;
            ct.dst = dst;
            return Ok(Outcome::applied());
        }
        if let Some(&rid) = self.model.rel_names.get(name) {
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("`{name}` is defined as a relation type, not a connection"),
            )
            .with_ref("rel", name, Some(rid.raw()))
            .with_actual(self.model.rel_statement(&self.model.rels[&rid]).to_value()));
        }
        Err(self.unknown("conn", name).with_hint(
            Statement::Define(conn_def(name, directed, source, carrier, target)).to_value(),
        ))
    }

    // ---- edges -----------------------------------------------------------------

    fn shape_error(&self, ty: &str, slot: &str, pat: &Pattern, node: NodeId) -> LangError {
        let path = self.model.node_path(node);
        LangError::new(
            ErrorCode::ShapeViolation,
            format!("{slot} {path} does not match the {slot} pattern of `{ty}`"),
        )
        .with_ref("slot", slot.to_string(), None)
        .with_ref("node", path.clone(), Some(node.raw()))
        .with_expected(
            serde_json::to_value(self.model.pattern_expr(pat)).expect("patterns serialize"),
        )
        .with_actual(Value::String(path))
    }

    fn check_ends_shape(
        &self,
        ty: &str,
        directed: bool,
        sp: &Pattern,
        dp: &Pattern,
        a: NodeId,
        b: NodeId,
    ) -> Result<(), LangError> {
        let m = &self.model;
        let ok = (m.matches(sp, a) && m.matches(dp, b))
            || (!directed && m.matches(sp, b) && m.matches(dp, a));
        if ok {
            return Ok(());
        }
        if !m.matches(sp, a) {
            Err(self.shape_error(ty, "source", sp, a))
        } else {
            Err(self.shape_error(ty, "target", dp, b))
        }
    }

    fn extend_views(&mut self, eid: EdgeId, views: BTreeSet<ViewId>) -> Outcome {
        let e = self.model.edges.get_mut(&eid).expect("edge exists");
        let before = e.views.len();
        e.views.extend(views);
        if e.views.len() > before {
            Outcome::applied()
        } else {
            Outcome::Noop
        }
    }

    fn insert_edge(&mut self, payload: EdgePayload, views: BTreeSet<ViewId>) {
        let id = EdgeId(self.model.alloc());
        self.model.edges.insert(id, Edge { id, payload, views });
    }

    fn do_rel_edge(
        &mut self,
        rel: &str,
        source: &str,
        target: &str,
        views: &[String],
    ) -> Result<Outcome, LangError> {
        let rel_id = *self.model.rel_names.get(rel).ok_or_else(|| {
            let mut e = self.unknown("rel", rel);
            if self.model.conn_names.contains_key(rel) {
                e.message =
                    format!("`{rel}` is a connection type; connections attach through ports");
            }
            e
        })?;
        let a = self.resolve_abs(source)?;
        let b = self.resolve_abs(target)?;
        let view_ids = self.resolve_views(views)?;
        let rt = self.model.rels[&rel_id].clone();
        self.check_ends_shape(&rt.name, rt.directed, &rt.src, &rt.dst, a, b)?;
        if let Some(eid) = self.model.find_rel_edge(rel_id, a, b) {
            return Ok(self.extend_views(eid, view_ids));
        }
        self.insert_edge(
            EdgePayload::Rel {
                rel: rel_id,
                src: a,
                dst: b,
            },
            view_ids,
        );
        Ok(Outcome::applied())
    }

    /// Look up a port by name on a node, enforcing that its connection type
    /// and side, fixed at first use, agree with this use. `Ok(None)` means the
    /// port does not exist yet.
    fn lookup_port(
        &self,
        node: NodeId,
        name: &str,
        conn: ConnId,
        side: Option<Side>,
    ) -> Result<Option<PortId>, LangError> {
        let Some(&pid) = self.model.nodes[&node].ports.get(name) else {
            return Ok(None);
        };
        let p = &self.model.ports[&pid];
        if p.conn != conn {
            return Err(LangError::new(
                ErrorCode::PortTypeConflict,
                format!(
                    "port {} is fixed to connection type `{}` by its first use",
                    self.model.port_path(pid),
                    self.model.conns[&p.conn].name
                ),
            )
            .with_ref("port", self.model.port_path(pid), Some(pid.raw()))
            .with_expected(Value::String(self.model.conns[&p.conn].name.clone()))
            .with_actual(Value::String(self.model.conns[&conn].name.clone()))
            .with_hint(json!({ "stmt": "query", "scopes": [self.model.node_path(node)] })));
        }
        if let (Some(want), Some(have)) = (side, p.side)
            && want != have
        {
            return Err(LangError::new(
                ErrorCode::PortSideConflict,
                format!(
                    "port {} is fixed to the {} side by its first use",
                    self.model.port_path(pid),
                    have.describe()
                ),
            )
            .with_ref("port", self.model.port_path(pid), Some(pid.raw()))
            .with_expected(Value::String(have.describe().into()))
            .with_actual(Value::String(want.describe().into()))
            .with_hint(json!({ "stmt": "query", "scopes": [self.model.node_path(node)] })));
        }
        Ok(Some(pid))
    }

    fn create_port(
        &mut self,
        node: NodeId,
        name: &str,
        conn: ConnId,
        side: Option<Side>,
    ) -> PortId {
        let id = PortId(self.model.alloc());
        self.model.ports.insert(
            id,
            Port {
                id,
                node,
                name: name.to_string(),
                conn,
                side,
            },
        );
        self.model
            .nodes
            .get_mut(&node)
            .expect("port owner exists")
            .ports
            .insert(name.to_string(), id);
        id
    }

    /// A port created under an undirected definition has no side; if the type
    /// was later redefined as directed, the next use fixes the side.
    fn fix_port_side(&mut self, pid: PortId, side: Option<Side>) {
        if let Some(want) = side {
            let p = self.model.ports.get_mut(&pid).expect("port exists");
            if p.side.is_none() {
                p.side = Some(want);
            }
        }
    }

    fn do_conn_edge(
        &mut self,
        conn: &str,
        source: &End,
        carrier: Option<&str>,
        target: &End,
        views: &[String],
    ) -> Result<Outcome, LangError> {
        let conn_id = *self.model.conn_names.get(conn).ok_or_else(|| {
            let mut e = self.unknown("conn", conn);
            if self.model.rel_names.contains_key(conn) {
                e.message = format!("`{conn}` is a relation type; relations relate whole nodes");
            }
            e
        })?;
        self.check_ident(&source.port, "port name")?;
        self.check_ident(&target.port, "port name")?;
        let a = self.resolve_abs(&source.node)?;
        let b = self.resolve_abs(&target.node)?;
        let view_ids = self.resolve_views(views)?;
        let ct = self.model.conns[&conn_id].clone();

        let carrier_node = match (&ct.carrier, carrier) {
            (Some(cp), None) => {
                return Err(LangError::new(
                    ErrorCode::CarrierRequired,
                    format!("`{conn}` is ternary: every instantiation names a carried node"),
                )
                .with_expected(
                    serde_json::to_value(self.model.pattern_expr(cp)).expect("patterns serialize"),
                ));
            }
            (None, Some(_)) => {
                return Err(LangError::new(
                    ErrorCode::CarrierForbidden,
                    format!("`{conn}` is binary: it never names a carrier"),
                ));
            }
            (Some(cp), Some(cpath)) => {
                let c = self.resolve_abs(cpath)?;
                if !self.model.matches(cp, c) {
                    return Err(self.shape_error(&ct.name, "carrier", cp, c));
                }
                Some(c)
            }
            (None, None) => None,
        };

        if self.model.nodes[&a].parent != self.model.nodes[&b].parent {
            return Err(LangError::new(
                ErrorCode::CrossScope,
                "connections operate between nodes at the same level; crossing a scope boundary is what applications are for",
            )
            .with_ref("node", self.model.node_path(a), Some(a.raw()))
            .with_ref("node", self.model.node_path(b), Some(b.raw())));
        }

        self.check_ends_shape(&ct.name, ct.directed, &ct.src, &ct.dst, a, b)?;

        let side_a = ct.directed.then_some(Side::Source);
        let side_b = ct.directed.then_some(Side::Target);
        let pa = self.lookup_port(a, &source.port, conn_id, side_a)?;
        let pb = self.lookup_port(b, &target.port, conn_id, side_b)?;

        if let (Some(pa), Some(pb)) = (pa, pb)
            && let Some(eid) = self.model.find_conn_edge(conn_id, pa, carrier_node, pb)
        {
            return Ok(self.extend_views(eid, view_ids));
        }
        let pa = match pa {
            Some(p) => {
                self.fix_port_side(p, side_a);
                p
            }
            None => self.create_port(a, &source.port, conn_id, side_a),
        };
        let pb = match pb {
            Some(p) => {
                self.fix_port_side(p, side_b);
                p
            }
            None => self.create_port(b, &target.port, conn_id, side_b),
        };
        self.insert_edge(
            EdgePayload::Conn {
                conn: conn_id,
                src_port: pa,
                carrier: carrier_node,
                dst_port: pb,
            },
            view_ids,
        );
        Ok(Outcome::applied())
    }

    fn do_app(
        &mut self,
        node: &str,
        port: &str,
        route: Option<&PatternExpr>,
        inner: &End,
    ) -> Result<Outcome, LangError> {
        let n = self.resolve_abs(node)?;
        self.check_ident(port, "port name")?;
        self.check_ident(&inner.port, "port name")?;
        if inner.node.contains('.') {
            return Err(LangError::new(
                ErrorCode::Parse,
                format!(
                    "the inner end of an application is a bare child name of `{node}`, got path `{}`",
                    inner.node
                ),
            ));
        }
        let node_path = self.model.node_path(n);
        let Some(&outer_pid) = self.model.nodes[&n].ports.get(port) else {
            return Err(LangError::new(
                ErrorCode::NoOuterPort,
                format!("no connection attaches a port `{port}` to {node_path}"),
            )
            .with_ref("port", format!("{node_path}.{port}"), None)
            .with_hint(json!({ "stmt": "query", "scopes": [node_path] })));
        };
        let qualifier = route.map(|r| self.resolve_pattern(r)).transpose()?;
        let inner_node = self
            .model
            .children(Some(n))
            .get(&inner.node)
            .copied()
            .ok_or_else(|| self.unknown("node", &format!("{node_path}.{}", inner.node)))?;
        let outer = self.model.ports[&outer_pid].clone();
        let existing_inner = self.lookup_port(inner_node, &inner.port, outer.conn, outer.side)?;
        if let Some(ip) = existing_inner
            && self
                .model
                .find_app_edge(outer_pid, &qualifier, ip)
                .is_some()
        {
            return Ok(Outcome::Noop);
        }
        for other in self.model.apps_on_outer_port(outer_pid) {
            let EdgePayload::App { qualifier: oq, .. } = &other.payload else {
                unreachable!("apps_on_outer_port yields applications");
            };
            let conflict = match (&qualifier, oq) {
                (None, None) => Some(None),
                (Some(q1), Some(q2)) => self
                    .model
                    .nodes
                    .keys()
                    .find(|x| self.model.matches(q1, **x) && self.model.matches(q2, **x))
                    .map(Some),
                _ => None,
            };
            if let Some(witness) = conflict {
                let other_stmt = self.model.edge_statement(other).to_value();
                let mut e = LangError::new(
                    ErrorCode::AmbiguousDelegation,
                    match witness {
                        Some(w) => format!(
                            "delegation would be ambiguous: an existing delegation already routes {}",
                            self.model.node_path(*w)
                        ),
                        None => "the port already has an unqualified delegation".to_string(),
                    },
                )
                .with_ref("edge", other_stmt.to_string(), Some(other.id.raw()))
                .with_actual(other_stmt)
                .with_hint(json!({ "stmt": "query", "scopes": [node_path.clone()] }));
                if let Some(w) = witness {
                    e = e.with_ref("node", self.model.node_path(*w), Some(w.raw()));
                }
                return Err(e);
            }
        }
        let ip = match existing_inner {
            Some(p) => {
                self.fix_port_side(p, outer.side);
                p
            }
            None => self.create_port(inner_node, &inner.port, outer.conn, outer.side),
        };
        self.insert_edge(
            EdgePayload::App {
                outer: outer_pid,
                qualifier,
                inner: ip,
            },
            BTreeSet::new(),
        );
        Ok(Outcome::applied())
    }

    // ---- mutations -----------------------------------------------------------

    fn do_rename(&mut self, node: &str, to: &str) -> Result<Outcome, LangError> {
        let id = self.resolve_abs(node)?;
        self.check_ident(to, "name")?;
        let old = self.model.nodes[&id].name.clone();
        if old == to {
            return Ok(Outcome::Noop);
        }
        let parent = self.model.nodes[&id].parent;
        if let Some(&other) = self.model.children(parent).get(to) {
            return Err(LangError::new(
                ErrorCode::DupName,
                format!("a sibling named `{to}` already exists"),
            )
            .with_ref("node", self.model.node_path(other), Some(other.raw())));
        }
        match parent {
            Some(p) => {
                let children = &mut self
                    .model
                    .nodes
                    .get_mut(&p)
                    .expect("parent exists")
                    .children;
                children.remove(&old);
                children.insert(to.to_string(), id);
            }
            None => {
                self.model.root.remove(&old);
                self.model.root.insert(to.to_string(), id);
            }
        }
        self.model.nodes.get_mut(&id).expect("node exists").name = to.to_string();
        Ok(Outcome::applied())
    }

    fn do_delete(
        &mut self,
        node: Option<&str>,
        edge: Option<&Statement>,
        rel: Option<&str>,
        conn: Option<&str>,
        view: Option<&str>,
    ) -> Result<Outcome, LangError> {
        let seed = if let Some(path) = node {
            Seed::Node(self.resolve_abs(path)?)
        } else if let Some(e) = edge {
            Seed::Edge(self.find_edge_stmt(e)?)
        } else if let Some(name) = rel {
            let id = *self
                .model
                .rel_names
                .get(name)
                .ok_or_else(|| self.unknown("rel", name))?;
            if self.model.rels[&id].stdlib {
                return Err(LangError::new(
                    ErrorCode::StdlibProtected,
                    format!("`{name}` is a stdlib relation and cannot be deleted"),
                ));
            }
            Seed::Rel(id)
        } else if let Some(name) = conn {
            Seed::Conn(
                *self
                    .model
                    .conn_names
                    .get(name)
                    .ok_or_else(|| self.unknown("conn", name))?,
            )
        } else if let Some(name) = view {
            Seed::View(
                *self
                    .model
                    .view_names
                    .get(name)
                    .ok_or_else(|| self.unknown("view", name))?,
            )
        } else {
            // Exactly-one-target is enforced at parse time.
            return Err(LangError::new(
                ErrorCode::Parse,
                "`delete` takes exactly one target",
            ));
        };
        let cascade = cascade::delete(&mut self.model, seed);
        Ok(Outcome::Applied {
            cascade: Some(cascade),
        })
    }

    fn do_untag(&mut self, edge: &Statement, views: &[String]) -> Result<Outcome, LangError> {
        let eid = self.find_edge_stmt(edge)?;
        let view_ids = self.resolve_views(views)?;
        let e = self.model.edges.get_mut(&eid).expect("edge exists");
        let before = e.views.len();
        for v in &view_ids {
            e.views.remove(v);
        }
        if e.views.len() < before {
            Ok(Outcome::applied())
        } else {
            Ok(Outcome::Noop)
        }
    }

    /// Resolve an edge addressed structurally, by restating it. A `views`
    /// field inside the restatement is ignored — views are not part of edge
    /// identity.
    fn find_edge_stmt(&self, edge: &Statement) -> Result<EdgeId, LangError> {
        let no_such_edge = || {
            LangError::new(ErrorCode::UnknownName, "no such edge")
                .with_ref("edge", edge.pseudo(), None)
                .with_hint(json!({ "stmt": "query" }))
        };
        match edge {
            Statement::RelEdge {
                rel,
                source,
                target,
                ..
            } => {
                let r = *self
                    .model
                    .rel_names
                    .get(rel)
                    .ok_or_else(|| self.unknown("rel", rel))?;
                let a = self.resolve_abs(source)?;
                let b = self.resolve_abs(target)?;
                self.model.find_rel_edge(r, a, b).ok_or_else(no_such_edge)
            }
            Statement::ConnEdge {
                conn,
                source,
                carrier,
                target,
                ..
            } => {
                let c = *self
                    .model
                    .conn_names
                    .get(conn)
                    .ok_or_else(|| self.unknown("conn", conn))?;
                let a = self.resolve_abs(&source.node)?;
                let b = self.resolve_abs(&target.node)?;
                let pa = *self.model.nodes[&a]
                    .ports
                    .get(&source.port)
                    .ok_or_else(|| {
                        self.unknown(
                            "port",
                            &format!("{}.{}", self.model.node_path(a), source.port),
                        )
                    })?;
                let pb = *self.model.nodes[&b]
                    .ports
                    .get(&target.port)
                    .ok_or_else(|| {
                        self.unknown(
                            "port",
                            &format!("{}.{}", self.model.node_path(b), target.port),
                        )
                    })?;
                let carrier_node = carrier.as_ref().map(|p| self.resolve_abs(p)).transpose()?;
                self.model
                    .find_conn_edge(c, pa, carrier_node, pb)
                    .ok_or_else(no_such_edge)
            }
            Statement::App {
                node,
                port,
                route,
                inner,
            } => {
                let n = self.resolve_abs(node)?;
                let node_path = self.model.node_path(n);
                let outer = *self.model.nodes[&n]
                    .ports
                    .get(port)
                    .ok_or_else(|| self.unknown("port", &format!("{node_path}.{port}")))?;
                let qualifier = route
                    .as_ref()
                    .map(|r| self.resolve_pattern(r))
                    .transpose()?;
                let inner_node = self
                    .model
                    .children(Some(n))
                    .get(&inner.node)
                    .copied()
                    .ok_or_else(|| self.unknown("node", &format!("{node_path}.{}", inner.node)))?;
                let ip = *self.model.nodes[&inner_node]
                    .ports
                    .get(&inner.port)
                    .ok_or_else(|| {
                        self.unknown(
                            "port",
                            &format!("{}.{}", self.model.node_path(inner_node), inner.port),
                        )
                    })?;
                self.model
                    .find_app_edge(outer, &qualifier, ip)
                    .ok_or_else(no_such_edge)
            }
            // Non-edge kinds inside `edge` are rejected at parse time.
            _ => Err(LangError::new(
                ErrorCode::Parse,
                "`edge` must restate an edge statement",
            )),
        }
    }
}

/// An owned rel definition, for rendering runnable hints.
fn rel_def(
    name: &str,
    trans: bool,
    directed: bool,
    source: &PatternExpr,
    target: &PatternExpr,
) -> Definition {
    Definition::Rel {
        name: name.to_string(),
        trans,
        directed,
        source: source.clone(),
        target: target.clone(),
    }
}

/// An owned conn definition, for rendering runnable hints.
fn conn_def(
    name: &str,
    directed: bool,
    source: &PatternExpr,
    carrier: Option<&PatternExpr>,
    target: &PatternExpr,
) -> Definition {
    Definition::Conn {
        name: name.to_string(),
        directed,
        source: source.clone(),
        carrier: carrier.cloned(),
        target: target.clone(),
    }
}

fn bad_request(message: impl Into<String>) -> LangError {
    LangError::new(ErrorCode::BadRequest, message)
}

fn parse_request(value: &Value) -> Result<Request, LangError> {
    let obj = value
        .as_object()
        .ok_or_else(|| bad_request("a request is a JSON object"))?;
    for key in obj.keys() {
        if !matches!(key.as_str(), "statements" | "expect_revision" | "dry_run") {
            return Err(bad_request(format!("unknown request field `{key}`")));
        }
    }
    let statements = obj
        .get("statements")
        .ok_or_else(|| bad_request("missing `statements`"))?
        .as_array()
        .ok_or_else(|| bad_request("`statements` must be an array"))?
        .clone();
    let expect_revision = match obj.get("expect_revision") {
        None => None,
        Some(v) => Some(
            v.as_u64()
                .ok_or_else(|| bad_request("`expect_revision` must be a non-negative integer"))?,
        ),
    };
    let dry_run = match obj.get("dry_run") {
        None => false,
        Some(v) => v
            .as_bool()
            .ok_or_else(|| bad_request("`dry_run` must be a boolean"))?,
    };
    Ok(Request {
        statements,
        expect_revision,
        dry_run,
    })
}
