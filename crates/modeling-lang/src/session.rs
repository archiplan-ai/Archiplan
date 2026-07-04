//! The statement engine: parses input and applies it to a model.
//!
//! Every statement either applies, is an identical-restatement no-op, or fails
//! with a structured error that leaves the model untouched. A batch submitted
//! as one request is atomic; interactive statements apply one at a time.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::ast::{EdgeStmt, Path, PatternAst, Stmt, StmtKind};
use crate::cascade::{self, Seed};
use crate::error::{ErrorCode, LangError};
use crate::ids::{ConnId, EdgeId, NodeId, PortId, ViewId};
use crate::model::{
    ConnType, Edge, EdgePayload, Model, Node, Pattern, Port, RelType, Side, ViewDef,
};
use crate::parser::Parser;
use crate::query;
use crate::result::{BatchError, InteractiveResult, Outcome, StatementResult};

/// A modeling session: a [`Model`] plus the current scope, driven by the
/// textual statement API.
#[derive(Clone, Debug)]
pub struct Session {
    model: Model,
    scope: Vec<NodeId>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    /// A fresh session on an empty model with the standard library loaded.
    pub fn new() -> Self {
        Session {
            model: Model::new_with_stdlib(),
            scope: Vec::new(),
        }
    }

    /// Read access to the model.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// The current scope as an absolute dot path; empty at the root.
    pub fn scope_path(&self) -> String {
        match self.scope.last() {
            Some(n) => self.model.node_path(*n),
            None => String::new(),
        }
    }

    /// Execute a batch atomically: any error rolls the whole batch back and
    /// reports the failing statement's index alongside the error.
    pub fn execute(&mut self, src: &str) -> Result<Vec<StatementResult>, BatchError> {
        let mut parser = match Parser::new(src) {
            Ok(p) => p,
            Err(error) => return Err(BatchError { index: 0, error }),
        };
        let mut stmts = Vec::new();
        loop {
            match parser.next_stmt() {
                Ok(Some(s)) => stmts.push(s),
                Ok(None) => break,
                Err(error) => {
                    return Err(BatchError {
                        index: stmts.len(),
                        error,
                    });
                }
            }
        }
        let checkpoint = self.clone();
        let mut results = Vec::new();
        for (index, stmt) in stmts.iter().enumerate() {
            let source = parser.snippet(stmt.span).to_string();
            match self.apply(stmt, src) {
                Ok(outcome) => results.push(StatementResult { source, outcome }),
                Err(mut error) => {
                    *self = checkpoint;
                    if error.subject.is_empty() {
                        error.subject = source;
                    }
                    return Err(BatchError { index, error });
                }
            }
        }
        Ok(results)
    }

    /// Execute statements one at a time: each statement applies or fails
    /// independently; a failed statement leaves the model unchanged and the
    /// rest continue. A parse error stops at the unparseable statement.
    pub fn execute_interactive(&mut self, src: &str) -> Vec<InteractiveResult> {
        let mut parser = match Parser::new(src) {
            Ok(p) => p,
            Err(e) => {
                return vec![InteractiveResult {
                    source: e.subject.clone(),
                    result: Err(e),
                }];
            }
        };
        let mut out = Vec::new();
        loop {
            match parser.next_stmt() {
                Ok(None) => break,
                Ok(Some(stmt)) => {
                    let source = parser.snippet(stmt.span).to_string();
                    let checkpoint = self.clone();
                    match self.apply(&stmt, src) {
                        Ok(outcome) => {
                            out.push(InteractiveResult {
                                source,
                                result: Ok(outcome),
                            });
                        }
                        Err(mut e) => {
                            *self = checkpoint;
                            if e.subject.is_empty() {
                                e.subject = source.clone();
                            }
                            out.push(InteractiveResult {
                                source,
                                result: Err(e),
                            });
                        }
                    }
                }
                Err(e) => {
                    out.push(InteractiveResult {
                        source: e.subject.clone(),
                        result: Err(e),
                    });
                    break;
                }
            }
        }
        out
    }

    fn apply(&mut self, stmt: &Stmt, src: &str) -> Result<Outcome, LangError> {
        match &stmt.kind {
            StmtKind::Node { name, block } => self.do_node(name, block.as_deref(), src),
            StmtKind::View { name } => self.do_view(name),
            StmtKind::RelDecl {
                trans,
                name,
                src: sp,
                directed,
                dst,
            } => self.do_rel_decl(*trans, name, sp, *directed, dst),
            StmtKind::ConnDecl {
                name,
                src: sp,
                carrier,
                directed,
                dst,
            } => self.do_conn_decl(name, sp, carrier.as_ref(), *directed, dst),
            StmtKind::Edge { edge, views } => self.do_edge(edge, views),
            StmtKind::Open { path } => self.do_open(path),
            StmtKind::Block { path, stmts } => self.do_block(path, stmts, src),
            StmtKind::Rename { path, new_name } => self.do_rename(path, new_name),
            StmtKind::DeleteNode { path } => self.do_delete_node(path),
            StmtKind::DeleteEdge { edge } => self.do_delete_edge(edge),
            StmtKind::DeleteRel { name } => self.do_delete_rel(name),
            StmtKind::DeleteConn { name } => self.do_delete_conn(name),
            StmtKind::DeleteView { name } => self.do_delete_view(name),
            StmtKind::Untag { edge, views } => self.do_untag(edge, views),
            StmtKind::Ports { path, views } => {
                let node = self.resolve_node(path)?;
                let filter = self.view_filter(views)?;
                Ok(Outcome::Statements(query::ports(
                    &self.model,
                    node,
                    filter.as_ref(),
                )))
            }
            StmtKind::Check { views } => {
                let filter = self.view_filter(views)?;
                Ok(Outcome::Findings(query::check(
                    &self.model,
                    filter.as_ref(),
                )))
            }
            StmtKind::Dump { views } => {
                let filter = self.view_filter(views)?;
                Ok(Outcome::Statements(query::dump(
                    &self.model,
                    filter.as_ref(),
                )))
            }
        }
    }

    // ---- name resolution -------------------------------------------------

    fn unknown(&self, kind: &'static str, what: &str) -> LangError {
        LangError::new(ErrorCode::UnknownName, format!("unknown {kind} `{what}`"))
            .with_ref(kind, what, None)
            .with_hint("dump")
    }

    fn resolve_node(&self, path: &Path) -> Result<NodeId, LangError> {
        self.model
            .resolve_path(&self.scope, &path.segs)
            .ok_or_else(|| self.unknown("node", &path.to_string()))
    }

    fn resolve_pattern(&self, ast: &PatternAst) -> Result<Pattern, LangError> {
        Ok(match ast {
            PatternAst::Any => Pattern::Any,
            PatternAst::Exact(p) => Pattern::Exact(self.resolve_node(p)?),
            PatternAst::Classified { anchor, rel } => {
                let a = self.resolve_node(anchor)?;
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
                self.model
                    .view_names
                    .get(n)
                    .copied()
                    .ok_or_else(|| self.unknown("view", n).with_hint(format!("view {n};")))
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

    // ---- write operations --------------------------------------------------

    fn do_node(
        &mut self,
        name: &str,
        block: Option<&[Stmt]>,
        src: &str,
    ) -> Result<Outcome, LangError> {
        let cur = self.scope.last().copied();
        let existing = self.model.children(cur).get(name).copied();
        let (node, node_outcome) = match existing {
            Some(id) => (id, Outcome::Noop),
            None => {
                let id = NodeId(self.model.alloc());
                self.model.nodes.insert(
                    id,
                    Node {
                        id,
                        name: name.to_string(),
                        parent: cur,
                        children: BTreeMap::new(),
                        ports: BTreeMap::new(),
                    },
                );
                match cur {
                    Some(p) => {
                        self.model
                            .nodes
                            .get_mut(&p)
                            .expect("scope node exists")
                            .children
                            .insert(name.to_string(), id);
                    }
                    None => {
                        self.model.root.insert(name.to_string(), id);
                    }
                }
                (id, Outcome::Applied)
            }
        };
        match block {
            None => Ok(node_outcome),
            Some(stmts) => {
                let mut results = vec![StatementResult {
                    source: format!("node {name}"),
                    outcome: node_outcome,
                }];
                results.extend(self.run_block(node, stmts, src)?);
                Ok(Outcome::Block(results))
            }
        }
    }

    fn run_block(
        &mut self,
        node: NodeId,
        stmts: &[Stmt],
        src: &str,
    ) -> Result<Vec<StatementResult>, LangError> {
        let chain = self.model.scope_chain(node);
        let saved = std::mem::replace(&mut self.scope, chain);
        let mut results = Vec::new();
        for st in stmts {
            let source = src[st.span.start..st.span.end].trim().to_string();
            match self.apply(st, src) {
                Ok(outcome) => results.push(StatementResult { source, outcome }),
                Err(mut e) => {
                    self.scope = saved;
                    self.fix_scope();
                    if e.subject.is_empty() {
                        e.subject = source;
                    }
                    return Err(e);
                }
            }
        }
        self.scope = saved;
        self.fix_scope();
        Ok(results)
    }

    fn do_view(&mut self, name: &str) -> Result<Outcome, LangError> {
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
        Ok(Outcome::Applied)
    }

    fn do_rel_decl(
        &mut self,
        trans: bool,
        name: &str,
        src: &PatternAst,
        directed: bool,
        dst: &PatternAst,
    ) -> Result<Outcome, LangError> {
        let src = self.resolve_pattern(src)?;
        let dst = self.resolve_pattern(dst)?;
        if let Some(&id) = self.model.rel_names.get(name) {
            let existing = &self.model.rels[&id];
            if existing.trans == trans
                && existing.directed == directed
                && existing.src == src
                && existing.dst == dst
            {
                return Ok(Outcome::Noop);
            }
            let rendered = self.model.render_rel_decl(existing);
            if existing.stdlib {
                return Err(LangError::new(
                    ErrorCode::StdlibProtected,
                    format!("`{name}` is a stdlib relation and cannot be redeclared divergently"),
                )
                .with_actual(rendered));
            }
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("rel `{name}` is already declared with a different definition"),
            )
            .with_actual(rendered));
        }
        if let Some(&cid) = self.model.conn_names.get(name) {
            let rendered = self.model.render_conn_decl(&self.model.conns[&cid]);
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("`{name}` is already declared as a connection type"),
            )
            .with_actual(rendered));
        }
        let id = crate::ids::RelId(self.model.alloc());
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
        Ok(Outcome::Applied)
    }

    fn do_conn_decl(
        &mut self,
        name: &str,
        src: &PatternAst,
        carrier: Option<&PatternAst>,
        directed: bool,
        dst: &PatternAst,
    ) -> Result<Outcome, LangError> {
        let src = self.resolve_pattern(src)?;
        let carrier = carrier.map(|c| self.resolve_pattern(c)).transpose()?;
        let dst = self.resolve_pattern(dst)?;
        if let Some(&id) = self.model.conn_names.get(name) {
            let existing = &self.model.conns[&id];
            if existing.directed == directed
                && existing.src == src
                && existing.carrier == carrier
                && existing.dst == dst
            {
                return Ok(Outcome::Noop);
            }
            let rendered = self.model.render_conn_decl(existing);
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("conn `{name}` is already declared with a different definition"),
            )
            .with_actual(rendered));
        }
        if let Some(&rid) = self.model.rel_names.get(name) {
            let rendered = self.model.render_rel_decl(&self.model.rels[&rid]);
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("`{name}` is already declared as a relation type"),
            )
            .with_actual(rendered));
        }
        let id = ConnId(self.model.alloc());
        self.model.conns.insert(
            id,
            ConnType {
                id,
                name: name.to_string(),
                directed,
                src,
                carrier,
                dst,
            },
        );
        self.model.conn_names.insert(name.to_string(), id);
        Ok(Outcome::Applied)
    }

    // ---- edges ---------------------------------------------------------

    fn do_edge(&mut self, edge: &EdgeStmt, views: &[String]) -> Result<Outcome, LangError> {
        match edge {
            EdgeStmt::Rel { src, rel, dst } => self.do_rel_edge(src, rel, dst, views),
            EdgeStmt::Conn {
                src,
                src_port,
                conn,
                carrier,
                dst,
                dst_port,
            } => self.do_conn_edge(src, src_port, conn, carrier.as_ref(), dst, dst_port, views),
            EdgeStmt::App {
                port,
                qualifier,
                inner,
                inner_port,
            } => self.do_app_edge(port, qualifier.as_ref(), inner, inner_port),
        }
    }

    fn shape_error(&self, ty: &str, slot: &str, pat: &Pattern, node: NodeId) -> LangError {
        let path = self.model.node_path(node);
        LangError::new(
            ErrorCode::ShapeViolation,
            format!("{slot} {path} does not match the {slot} pattern of `{ty}`"),
        )
        .with_ref("slot", slot.to_string(), None)
        .with_ref("node", path.clone(), Some(node.raw()))
        .with_expected(self.model.render_pattern(pat))
        .with_actual(path)
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
            Outcome::Applied
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
        src: &Path,
        rel: &str,
        dst: &Path,
        views: &[String],
    ) -> Result<Outcome, LangError> {
        let rel_id = *self.model.rel_names.get(rel).ok_or_else(|| {
            let mut e = self.unknown("rel", rel);
            if self.model.conn_names.contains_key(rel) {
                e = e.with_hint(format!(
                    "`{rel}` is a connection type; connections attach through ports: A(port) {rel} B(port)"
                ));
            }
            e
        })?;
        let a = self.resolve_node(src)?;
        let b = self.resolve_node(dst)?;
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
        Ok(Outcome::Applied)
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
            .with_expected(self.model.conns[&p.conn].name.clone())
            .with_actual(self.model.conns[&conn].name.clone())
            .with_hint(format!("ports {}", self.model.node_path(node))));
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
            .with_expected(have.describe())
            .with_actual(want.describe())
            .with_hint(format!("ports {}", self.model.node_path(node))));
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

    #[allow(clippy::too_many_arguments)]
    fn do_conn_edge(
        &mut self,
        src: &Path,
        src_port: &str,
        conn: &str,
        carrier: Option<&Path>,
        dst: &Path,
        dst_port: &str,
        views: &[String],
    ) -> Result<Outcome, LangError> {
        let conn_id = *self.model.conn_names.get(conn).ok_or_else(|| {
            let mut e = self.unknown("conn", conn);
            if self.model.rel_names.contains_key(conn) {
                e = e.with_hint(format!(
                    "`{conn}` is a relation type; relations relate whole nodes: A {conn} B"
                ));
            }
            e
        })?;
        let a = self.resolve_node(src)?;
        let b = self.resolve_node(dst)?;
        let view_ids = self.resolve_views(views)?;
        let ct = self.model.conns[&conn_id].clone();

        let carrier_node = match (&ct.carrier, carrier) {
            (Some(cp), None) => {
                return Err(LangError::new(
                    ErrorCode::CarrierRequired,
                    format!("`{conn}` is ternary: every instantiation names a carried node"),
                )
                .with_expected(self.model.render_pattern(cp))
                .with_hint(format!(
                    "{src}({src_port}) {conn}(<carrier>) {dst}({dst_port})"
                )));
            }
            (None, Some(_)) => {
                return Err(LangError::new(
                    ErrorCode::CarrierForbidden,
                    format!("`{conn}` is binary: it never names a carrier"),
                )
                .with_hint(format!("{src}({src_port}) {conn} {dst}({dst_port})")));
            }
            (Some(cp), Some(cpath)) => {
                let c = self.resolve_node(cpath)?;
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
        let pa = self.lookup_port(a, src_port, conn_id, side_a)?;
        let pb = self.lookup_port(b, dst_port, conn_id, side_b)?;

        if let (Some(pa), Some(pb)) = (pa, pb)
            && let Some(eid) = self.model.find_conn_edge(conn_id, pa, carrier_node, pb)
        {
            return Ok(self.extend_views(eid, view_ids));
        }
        let pa = match pa {
            Some(p) => p,
            None => self.create_port(a, src_port, conn_id, side_a),
        };
        let pb = match pb {
            Some(p) => p,
            None => self.create_port(b, dst_port, conn_id, side_b),
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
        Ok(Outcome::Applied)
    }

    fn do_app_edge(
        &mut self,
        port: &str,
        qualifier: Option<&PatternAst>,
        inner: &Path,
        inner_port: &str,
    ) -> Result<Outcome, LangError> {
        let Some(&scope_node) = self.scope.last() else {
            return Err(LangError::new(
                ErrorCode::NoOuterPort,
                "an application delegates an enclosing node's port, but the current scope is the root",
            )
            .with_hint("open <node>, or write the application inside `<node> { ... }`"));
        };
        let scope_path = self.model.node_path(scope_node);
        let Some(&outer_pid) = self.model.nodes[&scope_node].ports.get(port) else {
            return Err(LangError::new(
                ErrorCode::NoOuterPort,
                format!("no connection attaches a port `{port}` to {scope_path}"),
            )
            .with_ref("port", format!("{scope_path}.{port}"), None)
            .with_hint(format!("ports {scope_path}")));
        };
        let qual = qualifier.map(|q| self.resolve_pattern(q)).transpose()?;
        let inner_node = self
            .model
            .resolve_in(Some(scope_node), &inner.segs)
            .ok_or_else(|| self.unknown("node", &format!("{scope_path}.{inner}")))?;
        if self.model.nodes[&inner_node].parent != Some(scope_node) {
            return Err(LangError::new(
                ErrorCode::CrossScope,
                format!(
                    "an application maps an outer port to a port of a direct inner node of {scope_path}; delegate through the intermediate scopes instead"
                ),
            )
            .with_ref("node", self.model.node_path(inner_node), Some(inner_node.raw())));
        }
        let outer = self.model.ports[&outer_pid].clone();
        let existing_inner = self.lookup_port(inner_node, inner_port, outer.conn, outer.side)?;
        if let Some(ip) = existing_inner
            && self.model.find_app_edge(outer_pid, &qual, ip).is_some()
        {
            return Ok(Outcome::Noop);
        }
        for other in self.model.apps_on_outer_port(outer_pid) {
            let EdgePayload::App { qualifier: oq, .. } = &other.payload else {
                unreachable!("apps_on_outer_port yields applications");
            };
            let conflict = match (&qual, oq) {
                (None, None) => Some(None),
                (Some(q1), Some(q2)) => self
                    .model
                    .nodes
                    .keys()
                    .find(|n| self.model.matches(q1, **n) && self.model.matches(q2, **n))
                    .map(Some),
                _ => None,
            };
            if let Some(witness) = conflict {
                let mut e = LangError::new(
                    ErrorCode::AmbiguousDelegation,
                    match witness {
                        Some(w) => format!(
                            "delegation would be ambiguous: {} already routes {}",
                            self.model.render_edge(other),
                            self.model.node_path(*w)
                        ),
                        None => format!(
                            "the port already has an unqualified delegation: {}",
                            self.model.render_edge(other)
                        ),
                    },
                )
                .with_ref("edge", self.model.render_edge(other), Some(other.id.raw()))
                .with_hint(format!("ports {scope_path}"));
                if let Some(w) = witness {
                    e = e.with_ref("node", self.model.node_path(*w), Some(w.raw()));
                }
                return Err(e);
            }
        }
        let ip = match existing_inner {
            Some(p) => p,
            None => self.create_port(inner_node, inner_port, outer.conn, outer.side),
        };
        self.insert_edge(
            EdgePayload::App {
                outer: outer_pid,
                qualifier: qual,
                inner: ip,
            },
            BTreeSet::new(),
        );
        Ok(Outcome::Applied)
    }

    // ---- scope ----------------------------------------------------------

    fn do_open(&mut self, path: &Path) -> Result<Outcome, LangError> {
        let node = self.resolve_node(path)?;
        let chain = self.model.scope_chain(node);
        if chain == self.scope {
            Ok(Outcome::Noop)
        } else {
            self.scope = chain;
            Ok(Outcome::Applied)
        }
    }

    fn do_block(&mut self, path: &Path, stmts: &[Stmt], src: &str) -> Result<Outcome, LangError> {
        let node = self.resolve_node(path)?;
        Ok(Outcome::Block(self.run_block(node, stmts, src)?))
    }

    /// Drop deleted nodes from the scope chain, keeping the surviving prefix.
    fn fix_scope(&mut self) {
        let valid = self
            .scope
            .iter()
            .take_while(|n| self.model.nodes.contains_key(n))
            .count();
        self.scope.truncate(valid);
    }

    // ---- mutation operations ---------------------------------------------

    fn do_rename(&mut self, path: &Path, new_name: &str) -> Result<Outcome, LangError> {
        let node = self.resolve_node(path)?;
        let old = self.model.nodes[&node].name.clone();
        if old == new_name {
            return Ok(Outcome::Noop);
        }
        let parent = self.model.nodes[&node].parent;
        if let Some(&other) = self.model.children(parent).get(new_name) {
            return Err(LangError::new(
                ErrorCode::DupName,
                format!("a sibling named `{new_name}` already exists"),
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
                children.insert(new_name.to_string(), node);
            }
            None => {
                self.model.root.remove(&old);
                self.model.root.insert(new_name.to_string(), node);
            }
        }
        self.model.nodes.get_mut(&node).expect("node exists").name = new_name.to_string();
        Ok(Outcome::Applied)
    }

    fn do_delete_node(&mut self, path: &Path) -> Result<Outcome, LangError> {
        let node = self.resolve_node(path)?;
        let cascade = cascade::delete(&mut self.model, Seed::Node(node));
        self.fix_scope();
        Ok(Outcome::Deleted { cascade })
    }

    fn do_delete_rel(&mut self, name: &str) -> Result<Outcome, LangError> {
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
        let cascade = cascade::delete(&mut self.model, Seed::Rel(id));
        Ok(Outcome::Deleted { cascade })
    }

    fn do_delete_conn(&mut self, name: &str) -> Result<Outcome, LangError> {
        let id = *self
            .model
            .conn_names
            .get(name)
            .ok_or_else(|| self.unknown("conn", name))?;
        let cascade = cascade::delete(&mut self.model, Seed::Conn(id));
        Ok(Outcome::Deleted { cascade })
    }

    fn do_delete_view(&mut self, name: &str) -> Result<Outcome, LangError> {
        let id = *self
            .model
            .view_names
            .get(name)
            .ok_or_else(|| self.unknown("view", name))?;
        let cascade = cascade::delete(&mut self.model, Seed::View(id));
        Ok(Outcome::Deleted { cascade })
    }

    fn do_delete_edge(&mut self, edge: &EdgeStmt) -> Result<Outcome, LangError> {
        let eid = self.find_edge_stmt(edge)?;
        let cascade = cascade::delete(&mut self.model, Seed::Edge(eid));
        Ok(Outcome::Deleted { cascade })
    }

    fn do_untag(&mut self, edge: &EdgeStmt, views: &[String]) -> Result<Outcome, LangError> {
        let eid = self.find_edge_stmt(edge)?;
        let view_ids = self.resolve_views(views)?;
        let e = self.model.edges.get_mut(&eid).expect("edge exists");
        let before = e.views.len();
        for v in &view_ids {
            e.views.remove(v);
        }
        if e.views.len() < before {
            Ok(Outcome::Applied)
        } else {
            Ok(Outcome::Noop)
        }
    }

    /// Resolve an edge addressed structurally, by restating it.
    fn find_edge_stmt(&self, edge: &EdgeStmt) -> Result<EdgeId, LangError> {
        let no_such_edge = || {
            LangError::new(ErrorCode::UnknownName, "no such edge")
                .with_ref("edge", "", None)
                .with_hint("dump")
        };
        match edge {
            EdgeStmt::Rel { src, rel, dst } => {
                let r = *self
                    .model
                    .rel_names
                    .get(rel)
                    .ok_or_else(|| self.unknown("rel", rel))?;
                let a = self.resolve_node(src)?;
                let b = self.resolve_node(dst)?;
                self.model.find_rel_edge(r, a, b).ok_or_else(no_such_edge)
            }
            EdgeStmt::Conn {
                src,
                src_port,
                conn,
                carrier,
                dst,
                dst_port,
            } => {
                let c = *self
                    .model
                    .conn_names
                    .get(conn)
                    .ok_or_else(|| self.unknown("conn", conn))?;
                let a = self.resolve_node(src)?;
                let b = self.resolve_node(dst)?;
                let pa = *self.model.nodes[&a].ports.get(src_port).ok_or_else(|| {
                    self.unknown("port", &format!("{}.{src_port}", self.model.node_path(a)))
                })?;
                let pb = *self.model.nodes[&b].ports.get(dst_port).ok_or_else(|| {
                    self.unknown("port", &format!("{}.{dst_port}", self.model.node_path(b)))
                })?;
                let carrier_node = carrier.as_ref().map(|p| self.resolve_node(p)).transpose()?;
                self.model
                    .find_conn_edge(c, pa, carrier_node, pb)
                    .ok_or_else(no_such_edge)
            }
            EdgeStmt::App {
                port,
                qualifier,
                inner,
                inner_port,
            } => {
                let Some(&scope_node) = self.scope.last() else {
                    return Err(LangError::new(
                        ErrorCode::NoOuterPort,
                        "application statements are scope-relative; open the delegating node first",
                    ));
                };
                let scope_path = self.model.node_path(scope_node);
                let outer = *self.model.nodes[&scope_node]
                    .ports
                    .get(port)
                    .ok_or_else(|| self.unknown("port", &format!("{scope_path}.{port}")))?;
                let qual = qualifier
                    .as_ref()
                    .map(|q| self.resolve_pattern(q))
                    .transpose()?;
                let inner_node = self
                    .model
                    .resolve_in(Some(scope_node), &inner.segs)
                    .ok_or_else(|| self.unknown("node", &format!("{scope_path}.{inner}")))?;
                let ip = *self.model.nodes[&inner_node]
                    .ports
                    .get(inner_port)
                    .ok_or_else(|| {
                        self.unknown(
                            "port",
                            &format!("{}.{inner_port}", self.model.node_path(inner_node)),
                        )
                    })?;
                self.model
                    .find_app_edge(outer, &qual, ip)
                    .ok_or_else(no_such_edge)
            }
        }
    }
}
