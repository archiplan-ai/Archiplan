//! The statement engine: applies batches of statements to a model.
//!
//! A batch is atomic: every statement either applies, is an
//! identical-restatement no-op, or fails with a structured error that rolls
//! the whole batch back. There is no session state — every statement is
//! absolutely addressed and self-contained.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::definition;
use crate::error::{ErrorCode, LangError};
use crate::ids::{ConnId, EdgeId, NodeId, PortId, RelId, ViewId};
use crate::model::{
    ConnType, Edge, EdgePayload, Model, Node, Pattern, Port, RelType, Side, ViewDef,
};
use crate::preset::Preset;
use crate::query;
use crate::result::{BatchError, Outcome, Request, Response};
use crate::statement::{Definition, End, PatternExpr, Statement, parse_statement};

/// A model plus the preset loaded as its standard library.
///
/// [`Workspace::execute`] runs a parsed batch; [`Workspace::handle`] speaks
/// the read-only request/response envelope of `archi/requirements/self-hosting/agents-read-lowered-statements.md`.
#[derive(Clone, Debug)]
pub struct Workspace {
    model: Model,
    preset: Preset,
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
    /// A fresh workspace on an empty model with the standard library loaded —
    /// the [`Preset::core`] preset, exactly the historical stdlib.
    pub fn new() -> Self {
        Self::with_preset(&Preset::core()).expect("the core preset loads")
    }

    /// A fresh workspace on an empty model with `preset` loaded as its
    /// standard library. Preset statements run through the ordinary engine;
    /// everything they create is sealed as stdlib: omitted from dumps and
    /// findings, excluded from analyses, and protected — tagging a stdlib
    /// edge into views is rejected.
    pub fn with_preset(preset: &Preset) -> Result<Self, LangError> {
        let mut ws = Workspace {
            model: Model::empty(),
            preset: preset.clone(),
        };
        ws.execute(preset.statements()).map_err(|b| {
            LangError::new(
                ErrorCode::PresetInvalid,
                format!(
                    "preset `{}` statement {} rejected — {}: {}",
                    preset.name(),
                    b.index,
                    b.error.code,
                    b.error.message
                ),
            )
            .with_subject(preset.statements()[b.index].to_value())
        })?;
        ws.model.seal_preset(preset.name())?;
        Ok(ws)
    }

    /// Rebuild a workspace by loading `preset` and replaying `statements`
    /// (typically a dump).
    pub fn restore(preset: &Preset, statements: &[Statement]) -> Result<Self, LangError> {
        let mut ws = Self::with_preset(preset)?;
        ws.execute(statements).map_err(|b| {
            let mut e = b.error;
            e.message = format!("statement {}: {}", b.index, e.message);
            e
        })?;
        Ok(ws)
    }

    /// Read access to the model.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// The preset loaded as this workspace's standard library.
    pub fn preset(&self) -> &Preset {
        &self.preset
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

    /// Handle one read request envelope: `{ "statements": [...] }`. The
    /// agent interface is read-only — a model is edited as `.arch` source —
    /// so only `query` and `check` statements are accepted; anything else is
    /// a protocol error (`E_BAD_REQUEST`). Statement failures carry the
    /// failing index.
    pub fn handle(&mut self, request: &Value) -> Response {
        let req = match parse_request(request) {
            Ok(r) => r,
            Err(e) => return Response::fail(None, e),
        };
        let mut statements = Vec::with_capacity(req.statements.len());
        for (index, v) in req.statements.iter().enumerate() {
            match parse_statement(v) {
                Ok(s) => statements.push(s),
                Err(error) => return Response::fail(Some(index), error),
            }
        }
        if let Some(write) = statements
            .iter()
            .find(|s| !matches!(s, Statement::Query { .. } | Statement::Check { .. }))
        {
            let e = bad_request(
                "the agent interface is read-only — a model is edited as `.arch` source; \
                 only `query` and `check` statements are accepted",
            )
            .with_subject(write.to_value());
            return Response::fail(None, e);
        }
        match self.execute(&statements) {
            Ok(results) => Response::ok(results),
            Err(b) => Response::fail(Some(b.index), b.error),
        }
    }

    // ---- statement dispatch ------------------------------------------------

    fn apply(&mut self, stmt: &Statement) -> Result<Outcome, LangError> {
        match stmt {
            Statement::Define(def) => self.do_define(def),
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
                rev_carrier,
                target,
                views,
            } => self.do_conn_edge(
                conn,
                source,
                carrier.as_deref(),
                rev_carrier.as_deref(),
                target,
                views,
            ),
            Statement::App {
                node,
                port,
                route,
                inner,
            } => self.do_app(node, port, route.as_ref(), inner),
            Statement::Query {
                types,
                kinds,
                views,
                scopes,
                carriers,
                edge_types,
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
                    carriers: carriers.as_deref().map(resolve_all).transpose()?,
                    edge_types: edge_types
                        .as_deref()
                        .map(|ts| self.resolve_edge_types(ts))
                        .transpose()?,
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

    /// Resolve `edge_types` filter names against the defined rel and conn
    /// types.
    fn resolve_edge_types(&self, names: &[String]) -> Result<query::EdgeTypeFilter, LangError> {
        let mut filter = query::EdgeTypeFilter {
            rels: BTreeSet::new(),
            conns: BTreeSet::new(),
        };
        for name in names {
            if let Some(&id) = self.model.rel_names.get(name) {
                filter.rels.insert(id);
            } else if let Some(&id) = self.model.conn_names.get(name) {
                filter.conns.insert(id);
            } else {
                return Err(self.unknown("edge-type", name));
            }
        }
        Ok(filter)
    }

    // ---- definitions -----------------------------------------------------------

    fn do_define(&mut self, def: &Definition) -> Result<Outcome, LangError> {
        match def {
            Definition::Node {
                path,
                ports,
                doc,
                port_docs,
            } => self.define_node(path, ports.as_deref(), doc.as_deref(), port_docs.as_ref()),
            Definition::View { name, doc } => self.define_view(name, doc.as_deref()),
            Definition::Rel {
                name,
                trans,
                directed,
                source,
                target,
                doc,
            } => self.define_rel(name, *trans, *directed, source, target, doc.as_deref()),
            Definition::Conn {
                name,
                directed,
                source,
                carrier,
                rev_carrier,
                target,
                doc,
            } => self.define_conn(
                name,
                *directed,
                source,
                carrier.as_ref(),
                rev_carrier.as_ref(),
                target,
                doc.as_deref(),
            ),
        }
    }

    /// Normalize and validate definition prose entering through `execute` —
    /// the same shared rule the source attach pass and the statement schema
    /// apply, so no door stores what a render could not re-parse.
    fn check_doc(&self, doc: Option<&str>) -> Result<Option<String>, LangError> {
        match doc {
            None => Ok(None),
            Some(text) => {
                let normalized = definition::normalize(text);
                definition::validate(&normalized)
                    .map_err(|m| LangError::new(ErrorCode::Parse, m))?;
                Ok(Some(normalized))
            }
        }
    }

    fn define_node(
        &mut self,
        path: &str,
        ports: Option<&[String]>,
        doc: Option<&str>,
        port_docs: Option<&BTreeMap<String, String>>,
    ) -> Result<Outcome, LangError> {
        let segs = self.parse_path(path)?;
        if let Some(ps) = ports {
            for p in ps {
                self.check_ident(p, "port name")?;
            }
        }
        let doc = self.check_doc(doc)?;
        let port_docs = match port_docs {
            None => None,
            Some(pd) => {
                let Some(ps) = ports else {
                    return Err(LangError::new(
                        ErrorCode::Parse,
                        "`port_docs` requires `ports`: definitions attach to declared ports",
                    ));
                };
                let mut normalized = BTreeMap::new();
                for (port, text) in pd {
                    if !ps.contains(port) {
                        return Err(LangError::new(
                            ErrorCode::Parse,
                            format!("`port_docs` names `{port}`, which is not in `ports`"),
                        ));
                    }
                    normalized.insert(
                        port.clone(),
                        self.check_doc(Some(text))?.expect("doc is present"),
                    );
                }
                Some(normalized)
            }
        };
        let (name, prefix) = segs.split_last().expect("paths are non-empty");
        let parent = if prefix.is_empty() {
            None
        } else {
            Some(self.model.resolve_in(None, prefix).ok_or_else(|| {
                self.unknown("node", &prefix.join("."))
                    .with_ref("path", path, None)
            })?)
        };
        if let Some(&id) = self.model.children(parent).get(name) {
            // An omitted field makes no claim; a present one must restate
            // what is stored exactly — ports as a set, definitions verbatim.
            let redeclared = |what: &str| {
                LangError::new(
                    ErrorCode::Redeclared,
                    format!("node `{path}` is already defined with {what}"),
                )
                .with_ref("node", path, Some(id.raw()))
                .with_actual(self.model.node_statement(id).to_value())
            };
            if let Some(claim) = ports {
                let declared = self.model.declared_ports(id);
                let mut claim_sorted: Vec<String> = claim.to_vec();
                claim_sorted.sort();
                if claim_sorted != declared {
                    return Err(redeclared("different ports"));
                }
            }
            if let Some(d) = &doc
                && self.model.nodes[&id].doc.as_ref() != Some(d)
            {
                return Err(redeclared("a different definition"));
            }
            if let Some(pd) = &port_docs
                && *pd != self.model.declared_port_docs(id)
            {
                return Err(redeclared("different port definitions"));
            }
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
                doc,
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
        for p in ports.unwrap_or_default() {
            let port_doc = port_docs.as_ref().and_then(|m| m.get(p)).cloned();
            self.create_port(id, p, None, None, true, port_doc);
        }
        Ok(Outcome::Applied)
    }

    fn define_view(&mut self, name: &str, doc: Option<&str>) -> Result<Outcome, LangError> {
        self.check_ident(name, "view name")?;
        let doc = self.check_doc(doc)?;
        if let Some(&id) = self.model.view_names.get(name) {
            if let Some(d) = &doc
                && self.model.views[&id].doc.as_ref() != Some(d)
            {
                return Err(LangError::new(
                    ErrorCode::Redeclared,
                    format!("view `{name}` is already defined with a different definition"),
                )
                .with_ref("view", name, Some(id.raw()))
                .with_actual(self.model.view_statement(&self.model.views[&id]).to_value()));
            }
            return Ok(Outcome::Noop);
        }
        let id = ViewId(self.model.alloc());
        self.model.views.insert(
            id,
            ViewDef {
                id,
                name: name.to_string(),
                doc,
            },
        );
        self.model.view_names.insert(name.to_string(), id);
        Ok(Outcome::Applied)
    }

    fn define_rel(
        &mut self,
        name: &str,
        trans: bool,
        directed: bool,
        source: &PatternExpr,
        target: &PatternExpr,
        doc: Option<&str>,
    ) -> Result<Outcome, LangError> {
        self.check_ident(name, "relation type name")?;
        let doc = self.check_doc(doc)?;
        let src = self.resolve_pattern(source)?;
        let dst = self.resolve_pattern(target)?;
        if let Some(&id) = self.model.rel_names.get(name) {
            let existing = &self.model.rels[&id];
            let identical = existing.trans == trans
                && existing.directed == directed
                && existing.src == src
                && existing.dst == dst
                && (doc.is_none() || existing.doc == doc);
            if identical {
                return Ok(Outcome::Noop);
            }
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("rel `{name}` is already defined differently"),
            )
            .with_ref("rel", name, Some(id.raw()))
            .with_actual(self.model.rel_statement(existing).to_value()));
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
                doc,
            },
        );
        self.model.rel_names.insert(name.to_string(), id);
        Ok(Outcome::Applied)
    }

    /// An undirected type has no lanes to tell apart; a reverse carrier is
    /// meaningful only where direction (initiation) orients the lanes.
    fn check_rev_lane(
        &self,
        directed: bool,
        rev_carrier: Option<&PatternExpr>,
    ) -> Result<(), LangError> {
        if !directed && rev_carrier.is_some() {
            return Err(LangError::new(
                ErrorCode::Parse,
                "an undirected connection type has no lanes; `rev_carrier` requires `directed`",
            ));
        }
        Ok(())
    }

    fn define_conn(
        &mut self,
        name: &str,
        directed: bool,
        source: &PatternExpr,
        carrier: Option<&PatternExpr>,
        rev_carrier: Option<&PatternExpr>,
        target: &PatternExpr,
        doc: Option<&str>,
    ) -> Result<Outcome, LangError> {
        self.check_ident(name, "connection type name")?;
        self.check_rev_lane(directed, rev_carrier)?;
        let doc = self.check_doc(doc)?;
        let src = self.resolve_pattern(source)?;
        let carrier_pat = carrier.map(|c| self.resolve_pattern(c)).transpose()?;
        let rev_carrier_pat = rev_carrier.map(|c| self.resolve_pattern(c)).transpose()?;
        let dst = self.resolve_pattern(target)?;
        if let Some(&id) = self.model.conn_names.get(name) {
            let existing = &self.model.conns[&id];
            let identical = existing.directed == directed
                && existing.src == src
                && existing.carrier == carrier_pat
                && existing.rev_carrier == rev_carrier_pat
                && existing.dst == dst
                && (doc.is_none() || existing.doc == doc);
            if identical {
                return Ok(Outcome::Noop);
            }
            return Err(LangError::new(
                ErrorCode::Redeclared,
                format!("conn `{name}` is already defined differently"),
            )
            .with_ref("conn", name, Some(id.raw()))
            .with_actual(self.model.conn_statement(existing).to_value()));
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
                rev_carrier: rev_carrier_pat,
                dst,
                doc,
            },
        );
        self.model.conn_names.insert(name.to_string(), id);
        Ok(Outcome::Applied)
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

    /// Extend an existing edge's view set. Stdlib edges cannot be tagged:
    /// dumps omit them, so tags on them would not survive a replay.
    fn extend_views(&mut self, eid: EdgeId, views: BTreeSet<ViewId>) -> Result<Outcome, LangError> {
        if !views.is_empty() && self.model.is_stdlib(eid.raw()) {
            let stmt = self.model.edge_statement(&self.model.edges[&eid]);
            return Err(LangError::new(
                ErrorCode::StdlibProtected,
                "a stdlib edge cannot be tagged into views; tags on it would not survive a dump replay",
            )
            .with_ref("edge", stmt.pseudo(), Some(eid.raw())));
        }
        let e = self.model.edges.get_mut(&eid).expect("edge exists");
        let before = e.views.len();
        e.views.extend(views);
        if e.views.len() > before {
            Ok(Outcome::Applied)
        } else {
            Ok(Outcome::Noop)
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
            return self.extend_views(eid, view_ids);
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
    /// and side, fixed at its first use, agree with this use. `Ok(None)` means
    /// the port does not exist yet; a declared, never-used port has no type or
    /// side yet and agrees with anything.
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
        if let Some(have) = p.conn
            && have != conn
        {
            return Err(LangError::new(
                ErrorCode::PortTypeConflict,
                format!(
                    "port {} is fixed to connection type `{}` by its first use",
                    self.model.port_path(pid),
                    self.model.conns[&have].name
                ),
            )
            .with_ref("port", self.model.port_path(pid), Some(pid.raw()))
            .with_expected(Value::String(self.model.conns[&have].name.clone()))
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
        conn: Option<ConnId>,
        side: Option<Side>,
        declared: bool,
        doc: Option<String>,
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
                declared,
                doc,
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

    /// Fix what this use pins down and the port has not fixed yet: the
    /// connection type and side of a declared, never-used port.
    fn fix_port_use(&mut self, pid: PortId, conn: ConnId, side: Option<Side>) {
        let p = self.model.ports.get_mut(&pid).expect("port exists");
        if p.conn.is_none() {
            p.conn = Some(conn);
        }
        if p.side.is_none() {
            p.side = side;
        }
    }

    /// Check one lane of a connection edge against its type: a lane with a
    /// carried slot requires a named carrier matching the slot's pattern; a
    /// lane without one rejects a named carrier.
    fn check_lane_carrier(
        &self,
        ty: &str,
        slot: &'static str,
        lane: Option<&Pattern>,
        named: Option<&str>,
    ) -> Result<Option<NodeId>, LangError> {
        let way = if slot == "carrier" {
            "forward"
        } else {
            "reverse"
        };
        match (lane, named) {
            (Some(cp), None) => Err(LangError::new(
                ErrorCode::CarrierRequired,
                format!(
                    "`{ty}` carries a node on its {way} lane: every instantiation names `{slot}`"
                ),
            )
            .with_ref("slot", slot, None)
            .with_expected(
                serde_json::to_value(self.model.pattern_expr(cp)).expect("patterns serialize"),
            )),
            (None, Some(_)) => Err(LangError::new(
                ErrorCode::CarrierForbidden,
                format!("`{ty}` has no {way} carried slot: it never names `{slot}`"),
            )
            .with_ref("slot", slot, None)),
            (Some(cp), Some(cpath)) => {
                let c = self.resolve_abs(cpath)?;
                if !self.model.matches(cp, c) {
                    return Err(self.shape_error(ty, slot, cp, c));
                }
                Ok(Some(c))
            }
            (None, None) => Ok(None),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn do_conn_edge(
        &mut self,
        conn: &str,
        source: &End,
        carrier: Option<&str>,
        rev_carrier: Option<&str>,
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

        let carrier_node =
            self.check_lane_carrier(&ct.name, "carrier", ct.carrier.as_ref(), carrier)?;
        let rev_carrier_node = self.check_lane_carrier(
            &ct.name,
            "rev_carrier",
            ct.rev_carrier.as_ref(),
            rev_carrier,
        )?;

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
            && let Some(eid) =
                self.model
                    .find_conn_edge(conn_id, pa, carrier_node, rev_carrier_node, pb)
        {
            return self.extend_views(eid, view_ids);
        }
        let pa = match pa {
            Some(p) => {
                self.fix_port_use(p, conn_id, side_a);
                p
            }
            None => self.create_port(a, &source.port, Some(conn_id), side_a, false, None),
        };
        let pb = match pb {
            Some(p) => {
                self.fix_port_use(p, conn_id, side_b);
                p
            }
            None => self.create_port(b, &target.port, Some(conn_id), side_b, false, None),
        };
        self.insert_edge(
            EdgePayload::Conn {
                conn: conn_id,
                src_port: pa,
                carrier: carrier_node,
                rev_carrier: rev_carrier_node,
                dst_port: pb,
            },
            view_ids,
        );
        Ok(Outcome::Applied)
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
        // A declared port may exist unattached; delegation still needs an
        // attached use to inherit a connection type from.
        if !self.model.port_attached(outer_pid) {
            return Err(LangError::new(
                ErrorCode::NoOuterPort,
                format!(
                    "port `{port}` is declared on {node_path} but no connection attaches to it yet"
                ),
            )
            .with_ref("port", format!("{node_path}.{port}"), Some(outer_pid.raw()))
            .with_hint(json!({ "stmt": "query", "scopes": [node_path] })));
        }
        let qualifier = route.map(|r| self.resolve_pattern(r)).transpose()?;
        let inner_node = self
            .model
            .children(Some(n))
            .get(&inner.node)
            .copied()
            .ok_or_else(|| self.unknown("node", &format!("{node_path}.{}", inner.node)))?;
        let outer = self.model.ports[&outer_pid].clone();
        let outer_conn = outer.conn.expect("attached ports are typed");
        let existing_inner = self.lookup_port(inner_node, &inner.port, outer_conn, outer.side)?;
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
                self.fix_port_use(p, outer_conn, outer.side);
                p
            }
            None => self.create_port(inner_node, &inner.port, Some(outer_conn), outer.side, false, None),
        };
        self.insert_edge(
            EdgePayload::App {
                outer: outer_pid,
                qualifier,
                inner: ip,
            },
            BTreeSet::new(),
        );
        Ok(Outcome::Applied)
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
        if key.as_str() != "statements" {
            return Err(bad_request(format!("unknown request field `{key}`")));
        }
    }
    let statements = obj
        .get("statements")
        .ok_or_else(|| bad_request("missing `statements`"))?
        .as_array()
        .ok_or_else(|| bad_request("`statements` must be an array"))?
        .clone();
    Ok(Request { statements })
}
