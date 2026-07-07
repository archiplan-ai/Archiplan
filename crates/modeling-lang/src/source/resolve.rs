//! Resolution: from per-file ASTs to an absolute-path picture of the project.
//!
//! Three passes:
//!
//! 1. **Exports** (syntactic): each module's top-level definition names, per
//!    namespace, plus one-definition-site enforcement for type and view names.
//! 2. **Tree** (fixpoint): every `def node` — top-level, dotted, or nested in
//!    `def`/`open` blocks — lands at an absolute path; declared ports attach
//!    to their node. Order-independent: an `open` waits until its target
//!    exists, wherever it is defined.
//! 3. **Uses**: edges, applications and type-definition patterns resolve to
//!    absolute paths under the lexical rule — innermost block's children,
//!    enclosing blocks outward, then file scope (own defs, imports, preset).
//!
//! Visibility is per file and per namespace: referencing another module's
//! export requires importing it; preset names are ambient. Everything the
//! engine checks (shapes, sides, scopes) is deliberately *not* re-checked
//! here — the engine stays the semantic authority, and its errors are mapped
//! back to source via the lowering span table.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::Workspace;
use crate::statement::PatternExpr;

use super::ast::*;
use super::span::{Diagnostic, Span, Spanned};

/// One module: dotted path plus its parsed file.
pub(crate) struct ModuleAst {
    pub module: String,
    pub ast: FileAst,
}

/// Names the preset makes ambient, per namespace.
#[derive(Default, Clone, Debug)]
pub(crate) struct PresetInfo {
    pub node_paths: BTreeSet<String>,
    pub roots: BTreeSet<String>,
    pub rels: BTreeSet<String>,
    pub conns: BTreeSet<String>,
    pub views: BTreeSet<String>,
}

impl PresetInfo {
    /// Collect the ambient names from a workspace holding only the preset.
    pub(crate) fn from_workspace(ws: &Workspace) -> Self {
        let m = ws.model();
        let mut info = PresetInfo::default();
        for &id in m.root.values() {
            info.roots.insert(m.node_path(id));
        }
        for &id in m.nodes.keys().collect::<Vec<_>>() {
            info.node_paths.insert(m.node_path(id));
        }
        info.rels = m.rel_names.keys().cloned().collect();
        info.conns = m.conn_names.keys().cloned().collect();
        info.views = m.view_names.keys().cloned().collect();
        info
    }
}

/// A user-defined node, at its absolute path.
#[derive(Clone, Debug)]
pub(crate) struct NodeInfo {
    pub span: Span,
    /// Declared ports in declaration order.
    pub ports: Vec<String>,
    pub port_spans: BTreeMap<String, Span>,
}

#[derive(Clone, Debug)]
pub(crate) struct RelDefR {
    pub name: String,
    pub trans: bool,
    pub directed: bool,
    pub source: PatternExpr,
    pub target: PatternExpr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub(crate) struct ConnDefR {
    pub name: String,
    pub directed: bool,
    pub source: PatternExpr,
    pub fwd_carrier: Option<PatternExpr>,
    pub rev_carrier: Option<PatternExpr>,
    pub target: PatternExpr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub(crate) struct ViewDefR {
    pub name: String,
    pub span: Span,
}

/// A fully resolved edge, ready to lower.
#[derive(Clone, Debug)]
pub(crate) enum EdgeR {
    Rel {
        rel: String,
        source: String,
        target: String,
        views: Vec<String>,
        span: Span,
    },
    Conn {
        conn: String,
        source: (String, String),
        carrier: Option<String>,
        rev_carrier: Option<String>,
        target: (String, String),
        views: Vec<String>,
        span: Span,
    },
}

/// A fully resolved application.
#[derive(Clone, Debug)]
pub(crate) struct AppR {
    pub node: String,
    pub port: String,
    pub route: Option<PatternExpr>,
    pub inner_node: String,
    pub inner_port: String,
    pub span: Span,
}

/// The resolved project, in deterministic authoring order.
#[derive(Default)]
pub(crate) struct Resolution {
    pub nodes: BTreeMap<String, NodeInfo>,
    pub rels: Vec<RelDefR>,
    pub conns: Vec<ConnDefR>,
    pub views: Vec<ViewDefR>,
    pub edges: Vec<EdgeR>,
    pub apps: Vec<AppR>,
}

// ---- per-module scope -------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ns {
    Node,
    Rel,
    Conn,
    View,
}

impl Ns {
    fn describe(self) -> &'static str {
        match self {
            Ns::Node => "node",
            Ns::Rel => "rel",
            Ns::Conn => "conn",
            Ns::View => "view",
        }
    }
}

/// A module's top-level definition names — its exports.
#[derive(Default, Clone)]
struct Exports {
    roots: BTreeSet<String>,
    rels: BTreeSet<String>,
    conns: BTreeSet<String>,
    views: BTreeSet<String>,
}

impl Exports {
    fn of(&self, ns: Ns) -> &BTreeSet<String> {
        match ns {
            Ns::Node => &self.roots,
            Ns::Rel => &self.rels,
            Ns::Conn => &self.conns,
            Ns::View => &self.views,
        }
    }

    fn any(&self, name: &str) -> bool {
        self.roots.contains(name)
            || self.rels.contains(name)
            || self.conns.contains(name)
            || self.views.contains(name)
    }
}

/// What one module can see, per namespace: its own top-level names, imported
/// exports, and the ambient preset.
struct ModuleScope {
    exports: Exports,
    /// Fully imported modules.
    imports_all: Vec<usize>,
    /// Selectively imported names → source module.
    imports_named: BTreeMap<String, usize>,
}

pub(crate) struct Resolver<'a> {
    modules: &'a [ModuleAst],
    preset: &'a PresetInfo,
    scopes: Vec<ModuleScope>,
    module_index: BTreeMap<String, usize>,
    pub resolution: Resolution,
    pub diagnostics: Vec<Diagnostic>,
    /// Type/view definition sites, for duplicate detection: name → span.
    def_sites: BTreeMap<(&'static str, String), Span>,
}

/// Resolve the whole project. On any diagnostic the result is unusable.
pub(crate) fn resolve(
    modules: &[ModuleAst],
    preset: &PresetInfo,
) -> Result<Resolution, Vec<Diagnostic>> {
    let mut r = Resolver {
        modules,
        preset,
        scopes: Vec::new(),
        module_index: modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.module.clone(), i))
            .collect(),
        resolution: Resolution::default(),
        diagnostics: Vec::new(),
        def_sites: BTreeMap::new(),
    };
    r.collect_exports();
    r.resolve_imports();
    r.build_tree();
    if r.diagnostics.is_empty() {
        r.collect_type_defs();
        r.collect_uses();
    }
    if r.diagnostics.is_empty() {
        Ok(r.resolution)
    } else {
        Err(r.diagnostics)
    }
}

impl<'a> Resolver<'a> {
    // ---- pass 0: exports and definition sites ------------------------------

    fn collect_exports(&mut self) {
        for m in self.modules {
            let mut e = Exports::default();
            for item in &m.ast.items {
                match item {
                    Item::DefNode(d) if d.path.segments.len() == 1 => {
                        e.roots.insert(d.path.segments[0].value.clone());
                    }
                    Item::DefView { name } => {
                        e.views.insert(name.value.clone());
                    }
                    Item::DefRel { name, .. } => {
                        e.rels.insert(name.value.clone());
                    }
                    Item::DefConn { name, .. } => {
                        e.conns.insert(name.value.clone());
                    }
                    _ => {}
                }
            }
            self.scopes.push(ModuleScope {
                exports: e,
                imports_all: Vec::new(),
                imports_named: BTreeMap::new(),
            });
        }
        // One definition site per type/view name, project-wide; rel and conn
        // names share one namespace (they collide in the engine), and none
        // may capture a preset name.
        for m in self.modules {
            for item in &m.ast.items {
                let (kind, name): (&'static str, &Spanned<String>) = match item {
                    Item::DefRel { name, .. } => ("type", name),
                    Item::DefConn { name, .. } => ("type", name),
                    Item::DefView { name } => ("view", name),
                    _ => continue,
                };
                let preset_hit = match kind {
                    "type" => {
                        self.preset.rels.contains(&name.value)
                            || self.preset.conns.contains(&name.value)
                    }
                    _ => self.preset.views.contains(&name.value),
                };
                if preset_hit {
                    self.diagnostics.push(Diagnostic::new(
                        "E_REDECLARED",
                        format!(
                            "`{}` is defined by the preset; preset names cannot be redefined",
                            name.value
                        ),
                        name.span,
                    ));
                    continue;
                }
                match self.def_sites.entry((kind, name.value.clone())) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(name.span);
                    }
                    std::collections::btree_map::Entry::Occupied(o) => {
                        self.diagnostics.push(
                            Diagnostic::new(
                                "E_REDECLARED",
                                format!("`{}` is already defined", name.value),
                                name.span,
                            )
                            .with_note("first defined here", Some(*o.get())),
                        );
                    }
                }
            }
        }
    }

    // ---- pass 0b: imports ----------------------------------------------------

    fn resolve_imports(&mut self) {
        for (mi, m) in self.modules.iter().enumerate() {
            for imp in &m.ast.imports {
                let target = imp.module.render();
                let Some(&ti) = self.module_index.get(&target) else {
                    self.diagnostics.push(Diagnostic::new(
                        "E_UNKNOWN_MODULE",
                        format!("no module `{target}` in this project"),
                        imp.module.span,
                    ));
                    continue;
                };
                match &imp.only {
                    None => self.scopes[mi].imports_all.push(ti),
                    Some(names) => {
                        for n in names {
                            if !self.scopes[ti].exports.any(&n.value) {
                                self.diagnostics.push(Diagnostic::new(
                                    "E_UNKNOWN_NAME",
                                    format!("module `{target}` does not export `{}`", n.value),
                                    n.span,
                                ));
                                continue;
                            }
                            self.scopes[mi].imports_named.insert(n.value.clone(), ti);
                        }
                    }
                }
            }
        }
    }

    // ---- name lookup ----------------------------------------------------------

    /// Is `name` visible in `module`'s file scope, in namespace `ns`?
    fn visible(&self, module: usize, ns: Ns, name: &str) -> bool {
        let scope = &self.scopes[module];
        if scope.exports.of(ns).contains(name) {
            return true;
        }
        let preset_has = match ns {
            Ns::Node => self.preset.roots.contains(name),
            Ns::Rel => self.preset.rels.contains(name),
            Ns::Conn => self.preset.conns.contains(name),
            Ns::View => self.preset.views.contains(name),
        };
        if preset_has {
            return true;
        }
        if let Some(&ti) = scope.imports_named.get(name)
            && self.scopes[ti].exports.of(ns).contains(name)
        {
            return true;
        }
        scope
            .imports_all
            .iter()
            .any(|&ti| self.scopes[ti].exports.of(ns).contains(name))
    }

    /// The module defining `name` in namespace `ns`, if any — for the
    /// "import it" hint of E_NOT_VISIBLE.
    fn defining_module(&self, ns: Ns, name: &str) -> Option<&str> {
        self.scopes
            .iter()
            .position(|s| s.exports.of(ns).contains(name))
            .map(|i| self.modules[i].module.as_str())
    }

    fn unknown_or_hidden(&self, module: usize, ns: Ns, name: &str, span: Span) -> Diagnostic {
        match self.defining_module(ns, name) {
            Some(def_module) if def_module != self.modules[module].module => Diagnostic::new(
                "E_NOT_VISIBLE",
                format!(
                    "`{name}` is defined in module `{def_module}` — import it: `import {def_module}`"
                ),
                span,
            ),
            _ => Diagnostic::new(
                "E_UNKNOWN_NAME",
                format!("unknown {} `{name}`", ns.describe()),
                span,
            ),
        }
    }

    fn node_exists(&self, abs: &str) -> bool {
        self.resolution.nodes.contains_key(abs) || self.preset.node_paths.contains(abs)
    }

    /// Resolve a node path at a use site: first segment against the block
    /// chain (innermost first), then the file scope; the rest by descent.
    fn resolve_node_path(
        &self,
        module: usize,
        chain: &[String],
        path: &PathAst,
    ) -> Result<String, Diagnostic> {
        let first = &path.segments[0];
        let mut base: Option<String> = None;
        for scope in chain.iter().rev() {
            let candidate = format!("{scope}.{}", first.value);
            if self.node_exists(&candidate) {
                base = Some(candidate);
                break;
            }
        }
        let base = match base {
            Some(b) => b,
            None => {
                if !self.visible(module, Ns::Node, &first.value) {
                    return Err(self.unknown_or_hidden(module, Ns::Node, &first.value, first.span));
                }
                first.value.clone()
            }
        };
        let mut abs = base;
        for seg in &path.segments[1..] {
            abs = format!("{abs}.{}", seg.value);
        }
        if !self.node_exists(&abs) {
            return Err(Diagnostic::new(
                "E_UNKNOWN_NAME",
                format!("no node `{abs}`"),
                path.span,
            ));
        }
        Ok(abs)
    }

    // ---- pass 1: the node tree (fixpoint) ------------------------------------

    fn build_tree(&mut self) {
        enum Work<'w> {
            Def {
                module: usize,
                chain: Vec<String>,
                ast: &'w DefNodeAst,
            },
            Open {
                module: usize,
                chain: Vec<String>,
                ast: &'w OpenAst,
            },
        }

        let mut queue: VecDeque<Work<'a>> = VecDeque::new();
        for (mi, m) in self.modules.iter().enumerate() {
            for item in &m.ast.items {
                match item {
                    Item::DefNode(d) => queue.push_back(Work::Def {
                        module: mi,
                        chain: Vec::new(),
                        ast: d,
                    }),
                    Item::Open(o) => queue.push_back(Work::Open {
                        module: mi,
                        chain: Vec::new(),
                        ast: o,
                    }),
                    _ => {}
                }
            }
        }

        fn enqueue<'w>(
            queue: &mut VecDeque<Work<'w>>,
            module: usize,
            chain: &[String],
            scope: &str,
            body: &'w [BlockItem],
        ) {
            let mut inner = chain.to_vec();
            inner.push(scope.to_string());
            for item in body {
                match item {
                    BlockItem::DefNode(d) => queue.push_back(Work::Def {
                        module,
                        chain: inner.clone(),
                        ast: d,
                    }),
                    BlockItem::Open(o) => queue.push_back(Work::Open {
                        module,
                        chain: inner.clone(),
                        ast: o,
                    }),
                    _ => {}
                }
            }
        }

        let mut stuck: Vec<Work> = Vec::new();
        loop {
            let mut progressed = false;
            let mut retry: VecDeque<Work> = VecDeque::new();
            while let Some(work) = queue.pop_front() {
                match work {
                    Work::Def { module, chain, ast } => {
                        match self.try_def_container(module, &chain, ast) {
                            Ok(abs) => {
                                progressed = true;
                                self.insert_node(&abs, ast);
                                enqueue(&mut queue, module, &chain, &abs, &ast.body);
                            }
                            Err(_) => retry.push_back(Work::Def { module, chain, ast }),
                        }
                    }
                    Work::Open { module, chain, ast } => {
                        match self.resolve_node_path(module, &chain, &ast.path) {
                            Ok(target) => {
                                progressed = true;
                                enqueue(&mut queue, module, &chain, &target, &ast.body);
                            }
                            Err(_) => retry.push_back(Work::Open { module, chain, ast }),
                        }
                    }
                }
            }
            if retry.is_empty() {
                break;
            }
            if !progressed {
                stuck.extend(retry);
                break;
            }
            queue = retry;
        }

        for work in stuck {
            let d = match work {
                Work::Def { module, chain, ast } => self
                    .try_def_container(module, &chain, ast)
                    .expect_err("stuck work fails"),
                Work::Open { module, chain, ast } => self
                    .resolve_node_path(module, &chain, &ast.path)
                    .expect_err("stuck work fails"),
            };
            self.diagnostics.push(d);
        }
    }

    /// Absolute path a `def node` lands at, or why not (yet).
    fn try_def_container(
        &self,
        module: usize,
        chain: &[String],
        ast: &DefNodeAst,
    ) -> Result<String, Diagnostic> {
        let segs = &ast.path.segments;
        if segs.len() == 1 {
            return Ok(match chain.last() {
                Some(scope) => format!("{scope}.{}", segs[0].value),
                None => segs[0].value.clone(),
            });
        }
        let prefix = PathAst {
            segments: segs[..segs.len() - 1].to_vec(),
            span: ast.path.span,
        };
        let container = self.resolve_node_path(module, chain, &prefix)?;
        Ok(format!(
            "{container}.{}",
            segs.last().expect("nonempty").value
        ))
    }

    fn insert_node(&mut self, abs: &str, ast: &DefNodeAst) {
        if self.preset.node_paths.contains(abs) {
            self.diagnostics.push(Diagnostic::new(
                "E_REDECLARED",
                format!("`{abs}` is defined by the preset; preset names cannot be redefined"),
                ast.path.span,
            ));
            return;
        }
        if let Some(existing) = self.resolution.nodes.get(abs) {
            let first = existing.span;
            self.diagnostics.push(
                Diagnostic::new(
                    "E_REDECLARED",
                    format!("node `{abs}` is already defined — one definition site per node"),
                    ast.path.span,
                )
                .with_note("first defined here", Some(first)),
            );
            return;
        }
        let mut info = NodeInfo {
            span: ast.path.span,
            ports: Vec::new(),
            port_spans: BTreeMap::new(),
        };
        for item in &ast.body {
            if let BlockItem::Port(p) = item {
                if info.port_spans.contains_key(&p.value) {
                    self.diagnostics.push(Diagnostic::new(
                        "E_REDECLARED",
                        format!("port `{}` is declared twice on `{abs}`", p.value),
                        p.span,
                    ));
                    continue;
                }
                info.ports.push(p.value.clone());
                info.port_spans.insert(p.value.clone(), p.span);
            }
        }
        self.resolution.nodes.insert(abs.to_string(), info);
    }

    // ---- pass 2a: the def table ---------------------------------------------
    //
    // Every module's rel/conn/view defs land in the resolution before any
    // use binds, so what a use resolves to — carrier lanes above all — is a
    // function of the model's complete def set, never of module naming or
    // of a def's textual position relative to its uses
    // (`archi/requirements/self-hosting/uses-see-every-def.md`).

    fn collect_type_defs(&mut self) {
        for mi in 0..self.modules.len() {
            let items = &self.modules[mi].ast.items;
            for item in items {
                match item {
                    Item::DefView { name } => self.resolution.views.push(ViewDefR {
                        name: name.value.clone(),
                        span: name.span,
                    }),
                    Item::DefRel {
                        name,
                        trans,
                        directed,
                        source,
                        target,
                        span,
                    } => {
                        let (Some(source), Some(target)) =
                            (self.pattern(mi, &[], source), self.pattern(mi, &[], target))
                        else {
                            continue;
                        };
                        self.resolution.rels.push(RelDefR {
                            name: name.value.clone(),
                            trans: *trans,
                            directed: *directed,
                            source,
                            target,
                            span: *span,
                        });
                    }
                    Item::DefConn {
                        name,
                        source,
                        lanes,
                        target,
                        span,
                    } => {
                        let (Some(source), Some(target)) =
                            (self.pattern(mi, &[], source), self.pattern(mi, &[], target))
                        else {
                            continue;
                        };
                        let fwd_carrier = match &lanes.fwd_carrier {
                            Some(p) => match self.pattern(mi, &[], p) {
                                Some(p) => Some(p),
                                None => continue,
                            },
                            None => None,
                        };
                        let rev_carrier = match &lanes.rev_carrier {
                            Some(p) => match self.pattern(mi, &[], p) {
                                Some(p) => Some(p),
                                None => continue,
                            },
                            None => None,
                        };
                        self.resolution.conns.push(ConnDefR {
                            name: name.value.clone(),
                            directed: lanes.directed,
                            source,
                            fwd_carrier,
                            rev_carrier,
                            target,
                            span: *span,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    // ---- pass 2b: uses --------------------------------------------------------

    fn collect_uses(&mut self) {
        for mi in 0..self.modules.len() {
            let items = &self.modules[mi].ast.items;
            for item in items {
                match item {
                    Item::DefView { .. } | Item::DefRel { .. } | Item::DefConn { .. } => {}
                    Item::DefNode(d) => self.uses_in_def(mi, &mut Vec::new(), d),
                    Item::Open(o) => self.uses_in_open(mi, &mut Vec::new(), o),
                    Item::Edge(e) => self.edge(mi, &[], e),
                    Item::App(a) => self.app(mi, &[], a),
                }
            }
        }
    }

    fn uses_in_def(&mut self, module: usize, chain: &mut Vec<String>, ast: &DefNodeAst) {
        let Ok(abs) = self.try_def_container(module, chain, ast) else {
            return; // reported by build_tree
        };
        chain.push(abs);
        for item in &ast.body {
            self.uses_in_block_item(module, chain, item);
        }
        chain.pop();
    }

    fn uses_in_open(&mut self, module: usize, chain: &mut Vec<String>, ast: &OpenAst) {
        let Ok(target) = self.resolve_node_path(module, chain, &ast.path) else {
            return; // reported by build_tree
        };
        chain.push(target);
        for item in &ast.body {
            self.uses_in_block_item(module, chain, item);
        }
        chain.pop();
    }

    fn uses_in_block_item(&mut self, module: usize, chain: &mut Vec<String>, item: &BlockItem) {
        match item {
            BlockItem::Port(_) => {}
            BlockItem::DefNode(d) => self.uses_in_def(module, chain, d),
            BlockItem::Open(o) => self.uses_in_open(module, chain, o),
            BlockItem::Edge(e) => self.edge(module, &chain.clone(), e),
            BlockItem::App(a) => self.app(module, &chain.clone(), a),
        }
    }

    fn pattern(&mut self, module: usize, chain: &[String], p: &PatternAst) -> Option<PatternExpr> {
        match p {
            PatternAst::Any { .. } => Some(PatternExpr::Any),
            PatternAst::Exact { path } => match self.resolve_node_path(module, chain, path) {
                Ok(node) => Some(PatternExpr::Exact { node }),
                Err(d) => {
                    self.diagnostics.push(d);
                    None
                }
            },
            PatternAst::Classified { anchor, rel, .. } => {
                let anchor_abs = match self.resolve_node_path(module, chain, anchor) {
                    Ok(a) => a,
                    Err(d) => {
                        self.diagnostics.push(d);
                        return None;
                    }
                };
                if !self.visible(module, Ns::Rel, &rel.value) {
                    let d = self.unknown_or_hidden(module, Ns::Rel, &rel.value, rel.span);
                    self.diagnostics.push(d);
                    return None;
                }
                Some(PatternExpr::Classified {
                    anchor: anchor_abs,
                    rel: rel.value.clone(),
                })
            }
        }
    }

    /// The kind of an edge's type name, with visibility enforced.
    fn edge_type_kind(&mut self, module: usize, name: &Spanned<String>) -> Option<Ns> {
        for ns in [Ns::Rel, Ns::Conn] {
            if self.visible(module, ns, &name.value) {
                return Some(ns);
            }
        }
        let hidden_in = [Ns::Rel, Ns::Conn]
            .into_iter()
            .find(|&ns| self.defining_module(ns, &name.value).is_some());
        let d = match hidden_in {
            Some(ns) => self.unknown_or_hidden(module, ns, &name.value, name.span),
            None => Diagnostic::new(
                "E_UNKNOWN_NAME",
                format!("unknown type `{}`", name.value),
                name.span,
            ),
        };
        self.diagnostics.push(d);
        None
    }

    /// Split a conn end into node and port, enforcing the declare-first rule.
    fn conn_end(
        &mut self,
        module: usize,
        chain: &[String],
        path: &PathAst,
    ) -> Option<(String, String)> {
        if path.segments.len() < 2 {
            self.diagnostics.push(Diagnostic::new(
                "E_PARSE",
                "a connection end names a node and its port: `Node.port`",
                path.span,
            ));
            return None;
        }
        let node_path = PathAst {
            segments: path.segments[..path.segments.len() - 1].to_vec(),
            span: path.segments[0]
                .span
                .to(path.segments[path.segments.len() - 2].span),
        };
        let port = path.segments.last().expect("nonempty");
        let node = match self.resolve_node_path(module, chain, &node_path) {
            Ok(n) => n,
            Err(d) => {
                self.diagnostics.push(d);
                return None;
            }
        };
        self.check_port(&node, port)?;
        Some((node, port.value.clone()))
    }

    /// Every port used from source is declared at its node's definition.
    fn check_port(&mut self, node: &str, port: &Spanned<String>) -> Option<()> {
        match self.resolution.nodes.get(node) {
            Some(info) if info.port_spans.contains_key(&port.value) => Some(()),
            Some(info) => {
                let def_span = info.span;
                self.diagnostics.push(
                    Diagnostic::new(
                        "E_UNDECLARED_PORT",
                        format!("`{node}` declares no port `{}`", port.value),
                        port.span,
                    )
                    .with_note(format!("`{node}` is defined here"), Some(def_span)),
                );
                None
            }
            None => {
                // A preset node: it has no source definition to declare ports in.
                self.diagnostics.push(Diagnostic::new(
                    "E_UNDECLARED_PORT",
                    format!("`{node}` is a preset node; source files cannot attach ports to it"),
                    port.span,
                ));
                None
            }
        }
    }

    fn views_of(&mut self, module: usize, views: &[Spanned<String>]) -> Option<Vec<String>> {
        let mut out = Vec::new();
        for v in views {
            if !self.visible(module, Ns::View, &v.value) {
                let d = self.unknown_or_hidden(module, Ns::View, &v.value, v.span);
                self.diagnostics.push(d);
                return None;
            }
            out.push(v.value.clone());
        }
        Some(out)
    }

    fn edge(&mut self, module: usize, chain: &[String], e: &EdgeAst) {
        let Some(kind) = self.edge_type_kind(module, &e.type_name) else {
            return;
        };
        let Some(views) = self.views_of(module, &e.views) else {
            return;
        };
        match kind {
            Ns::Rel => {
                if let Some(c) = e.carriers.first() {
                    self.diagnostics.push(Diagnostic::new(
                        "E_CARRIER_FORBIDDEN",
                        format!(
                            "`{}` is a relation type; relations carry nothing",
                            e.type_name.value
                        ),
                        c.span,
                    ));
                    return;
                }
                let source = match self.resolve_node_path(module, chain, &e.lhs) {
                    Ok(n) => n,
                    Err(d) => {
                        self.diagnostics.push(d);
                        return;
                    }
                };
                let target = match self.resolve_node_path(module, chain, &e.rhs) {
                    Ok(n) => n,
                    Err(d) => {
                        self.diagnostics.push(d);
                        return;
                    }
                };
                self.resolution.edges.push(EdgeR::Rel {
                    rel: e.type_name.value.clone(),
                    source,
                    target,
                    views,
                    span: e.span,
                });
            }
            Ns::Conn => {
                let Some(source) = self.conn_end(module, chain, &e.lhs) else {
                    return;
                };
                let Some(target) = self.conn_end(module, chain, &e.rhs) else {
                    return;
                };
                let Some((carrier, rev_carrier)) = self.edge_carriers(module, chain, e) else {
                    return;
                };
                self.resolution.edges.push(EdgeR::Conn {
                    conn: e.type_name.value.clone(),
                    source,
                    carrier,
                    rev_carrier,
                    target,
                    views,
                    span: e.span,
                });
            }
            _ => unreachable!("edge types are rels or conns"),
        }
    }

    /// Map carrier arguments onto the type's lanes and infer omitted exact
    /// lanes. The engine re-checks arity and patterns; this produces fully
    /// explicit statements and catches mistakes with a source span.
    fn edge_carriers(
        &mut self,
        module: usize,
        chain: &[String],
        e: &EdgeAst,
    ) -> Option<(Option<String>, Option<String>)> {
        let def = self
            .resolution
            .conns
            .iter()
            .find(|c| c.name == e.type_name.value)
            .cloned();
        // Preset conns have no source def; args must then be fully tagged or
        // unambiguous — without lane info we pass a bare arg forward.
        let (fwd_lane, rev_lane) = match &def {
            Some(d) => (d.fwd_carrier.clone(), d.rev_carrier.clone()),
            None => (None, None),
        };

        let mut fwd_arg: Option<&CarrierArg> = None;
        let mut rev_arg: Option<&CarrierArg> = None;
        for arg in &e.carriers {
            match arg.dir {
                Some(LaneDir::Fwd) => fwd_arg = Some(arg),
                Some(LaneDir::Rev) => rev_arg = Some(arg),
                None => {
                    // A bare argument binds to the single carrying lane.
                    match (&fwd_lane, &rev_lane) {
                        (Some(_), None) | (None, None) => fwd_arg = Some(arg),
                        (None, Some(_)) => rev_arg = Some(arg),
                        (Some(_), Some(_)) => {
                            self.diagnostics.push(Diagnostic::new(
                                "E_PARSE",
                                format!(
                                    "both lanes of `{}` carry a node; tag the argument: `(->X)` or `(<-X)`",
                                    e.type_name.value
                                ),
                                arg.span,
                            ));
                            return None;
                        }
                    }
                }
            }
        }

        let mut fwd = match fwd_arg {
            None => None,
            Some(a) => match self.resolve_node_path(module, chain, &a.path) {
                Ok(n) => Some(n),
                Err(d) => {
                    self.diagnostics.push(d);
                    return None;
                }
            },
        };
        let mut rev = match rev_arg {
            None => None,
            Some(a) => match self.resolve_node_path(module, chain, &a.path) {
                Ok(n) => Some(n),
                Err(d) => {
                    self.diagnostics.push(d);
                    return None;
                }
            },
        };

        // Inference: an omitted lane whose pattern is an exact node defaults
        // to that node.
        if fwd.is_none()
            && let Some(PatternExpr::Exact { node }) = &fwd_lane
        {
            fwd = Some(node.clone());
        }
        if rev.is_none()
            && let Some(PatternExpr::Exact { node }) = &rev_lane
        {
            rev = Some(node.clone());
        }

        // A lane that still has a pattern but no carrier cannot lower to a
        // valid statement — report here, with the edge's span.
        if fwd.is_none() && fwd_lane.is_some() {
            self.diagnostics.push(Diagnostic::new(
                "E_CARRIER_REQUIRED",
                format!(
                    "`{}` carries a node on its forward lane: name it — `{}(->X)`",
                    e.type_name.value, e.type_name.value
                ),
                e.span,
            ));
            return None;
        }
        if rev.is_none() && rev_lane.is_some() {
            self.diagnostics.push(Diagnostic::new(
                "E_CARRIER_REQUIRED",
                format!(
                    "`{}` carries a node on its reverse lane: name it — `{}(<-X)`",
                    e.type_name.value, e.type_name.value
                ),
                e.span,
            ));
            return None;
        }
        Some((fwd, rev))
    }

    fn app(&mut self, module: usize, chain: &[String], a: &AppAst) {
        let (node, port) = if a.outer.segments.len() == 1 {
            let Some(scope) = chain.last() else {
                // The parser rejects bare-port apps at the top level.
                unreachable!("bare-port apps only occur inside blocks");
            };
            (scope.clone(), a.outer.segments[0].clone())
        } else {
            let node_path = PathAst {
                segments: a.outer.segments[..a.outer.segments.len() - 1].to_vec(),
                span: a.outer.span,
            };
            let node = match self.resolve_node_path(module, chain, &node_path) {
                Ok(n) => n,
                Err(d) => {
                    self.diagnostics.push(d);
                    return;
                }
            };
            (node, a.outer.segments.last().expect("nonempty").clone())
        };
        if self.check_port(&node, &port).is_none() {
            return;
        }
        let child = format!("{node}.{}", a.inner_node.value);
        if !self.node_exists(&child) {
            self.diagnostics.push(Diagnostic::new(
                "E_UNKNOWN_NAME",
                format!("`{node}` has no child `{}`", a.inner_node.value),
                a.inner_node.span,
            ));
            return;
        }
        if self.check_port(&child, &a.inner_port).is_none() {
            return;
        }
        let route = match &a.route {
            Some(r) => match self.pattern(module, chain, r) {
                Some(p) => Some(p),
                None => return,
            },
            None => None,
        };
        self.resolution.apps.push(AppR {
            node,
            port: port.value,
            route,
            inner_node: a.inner_node.value.clone(),
            inner_port: a.inner_port.value.clone(),
            span: a.span,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::Preset;
    use crate::source::parser::parse;
    use crate::source::span::SourceMap;

    fn try_resolve_with(
        preset: &Preset,
        sources: &[(&str, &str)],
    ) -> Result<Resolution, Vec<Diagnostic>> {
        let mut map = SourceMap::new();
        let mut modules = Vec::new();
        for (module, text) in sources {
            let fid = map.add_file(format!("src/{}.arch", module.replace('.', "/")), *text);
            let ast = parse(fid, text).map_err(|d| vec![d])?;
            modules.push(ModuleAst {
                module: module.to_string(),
                ast,
            });
        }
        modules.sort_by(|a, b| a.module.cmp(&b.module));
        let ws = Workspace::with_preset(preset).expect("preset loads");
        let preset = PresetInfo::from_workspace(&ws);
        resolve(&modules, &preset)
    }

    fn try_resolve(sources: &[(&str, &str)]) -> Result<Resolution, Vec<Diagnostic>> {
        try_resolve_with(&Preset::default_ontology(), sources)
    }

    fn resolve_ok(sources: &[(&str, &str)]) -> Resolution {
        match try_resolve(sources) {
            Ok(r) => r,
            Err(ds) => panic!(
                "resolution failed: {:?}",
                ds.iter().map(|d| format!("{d}")).collect::<Vec<_>>()
            ),
        }
    }

    fn codes(sources: &[(&str, &str)]) -> Vec<(String, String)> {
        match try_resolve(sources) {
            Ok(_) => panic!("expected diagnostics"),
            Err(ds) => ds
                .into_iter()
                .map(|d| (d.code.clone(), d.message.clone()))
                .collect(),
        }
    }

    /// The carriers of the single `conn` edge in a resolution.
    fn conn_carriers(r: &Resolution, name: &str) -> (Option<String>, Option<String>) {
        r.edges
            .iter()
            .find_map(|e| match e {
                EdgeR::Conn {
                    conn,
                    carrier,
                    rev_carrier,
                    ..
                } if conn == name => Some((carrier.clone(), rev_carrier.clone())),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no `{name}` edge resolved"))
    }

    // The next three tests pin `uses-see-every-def`
    // (`issues/carrier-inference-order-dependence.md`): binding is a
    // function of the model's complete def set, never of walk order.

    #[test]
    fn carrier_inference_reads_defs_from_later_modules() {
        // The conn def lives in the module sorting LAST; the edge
        // instantiating it, with both lanes inferred, in the module sorting
        // FIRST.
        let r = resolve_ok(&[
            (
                "aedge",
                "import zconns\ndef node A:\n  port drive\ndef node B:\n  port check\nA.drive invoke B.check\n",
            ),
            (
                "zconns",
                "import zmsgs\ndef conn invoke := * ->Command, <-Report *\n",
            ),
            ("zmsgs", "def node Command\ndef node Report\n"),
        ]);
        assert_eq!(
            conn_carriers(&r, "invoke"),
            (Some("Command".into()), Some("Report".into()))
        );

        // A genuinely uninferable lane still fails here, with the
        // compiler's hint at the edge — never downstream in the engine.
        let errs = codes(&[
            (
                "aedge",
                "import zconns\ndef node A:\n  port drive\ndef node B:\n  port check\nA.drive invoke B.check\n",
            ),
            ("zconns", "def conn invoke := * ->(Data type_of *) *\n"),
        ]);
        assert!(
            errs.iter()
                .any(|(c, m)| c == "E_CARRIER_REQUIRED" && m.contains("name it")),
            "{errs:?}"
        );
    }

    #[test]
    fn defs_bind_uses_that_precede_them_in_one_module() {
        let r = resolve_ok(&[(
            "solo",
            "def node Command\ndef node Report\ndef node A:\n  port drive\ndef node B:\n  port check\n\
             A.drive invoke B.check\ndef conn invoke := * ->Command, <-Report *\n",
        )]);
        assert_eq!(
            conn_carriers(&r, "invoke"),
            (Some("Command".into()), Some("Report".into()))
        );
    }

    #[test]
    fn defless_conns_keep_the_untyped_path() {
        // A conn the preset names but no source defines: the resolver has
        // no lane knowledge — a bare argument rides the forward lane,
        // absence stays absence, and the engine re-checks downstream. The
        // def sweep must change nothing here.
        let preset = Preset::from_value(
            "with-conn",
            &serde_json::json!([
                { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
                  "source": "*", "target": "*" },
                { "stmt": "define", "conn": "pipe", "directed": true,
                  "source": "*", "target": "*" }
            ]),
        )
        .expect("preset parses");
        let r = match try_resolve_with(
            &preset,
            &[(
                "solo",
                "def node X\ndef node A:\n  port p\n  port p2\ndef node B:\n  port q\n  port q2\n\
                 A.p pipe(X) B.q\nA.p2 pipe B.q2\n",
            )],
        ) {
            Ok(r) => r,
            Err(ds) => panic!("{ds:?}"),
        };
        let carriers: Vec<_> = r
            .edges
            .iter()
            .filter_map(|e| match e {
                EdgeR::Conn {
                    conn,
                    carrier,
                    rev_carrier,
                    ..
                } if conn == "pipe" => Some((carrier.clone(), rev_carrier.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(carriers, [(Some("X".into()), None), (None, None)]);
    }

    #[test]
    fn cross_file_defs_opens_and_flows_resolve() {
        let r = resolve_ok(&[
            (
                "auth",
                "def node AuthService:\n  port handle_login\nService type_of AuthService\n",
            ),
            (
                "auth_internals",
                "import auth\nopen AuthService:\n  def node Storage:\n    port save_cred_hash\n",
            ),
            (
                "ui",
                "import auth\nimport conns\ndef view login_flow\ndef node UI:\n  port login\nUI.login login AuthService.handle_login in login_flow\n",
            ),
            (
                "conns",
                "import messages\ndef conn login := * ->LoginForm, <-AuthResponse *\n",
            ),
            ("messages", "def node LoginForm\ndef node AuthResponse\n"),
        ]);
        assert!(r.nodes.contains_key("AuthService"));
        assert!(r.nodes.contains_key("AuthService.Storage"));
        assert_eq!(r.nodes["AuthService"].ports, ["handle_login"]);
        assert_eq!(r.nodes["AuthService.Storage"].ports, ["save_cred_hash"]);
        // The bidir conn edge inferred both exact-lane carriers.
        let conn_edge = r
            .edges
            .iter()
            .find_map(|e| match e {
                EdgeR::Conn {
                    conn,
                    source,
                    carrier,
                    rev_carrier,
                    target,
                    ..
                } if conn == "login" => Some((source, carrier, rev_carrier, target)),
                _ => None,
            })
            .expect("the login edge resolves");
        assert_eq!(conn_edge.0, &("UI".to_string(), "login".to_string()));
        assert_eq!(conn_edge.1.as_deref(), Some("LoginForm"));
        assert_eq!(conn_edge.2.as_deref(), Some("AuthResponse"));
        assert_eq!(
            conn_edge.3,
            &("AuthService".to_string(), "handle_login".to_string())
        );
        // The rel edge on the ambient preset relation resolved without imports.
        assert!(r.edges.iter().any(|e| matches!(
            e,
            EdgeR::Rel { rel, source, target, .. }
                if rel == "type_of" && source == "Service" && target == "AuthService"
        )));
    }

    #[test]
    fn cross_file_references_require_imports() {
        let errs = codes(&[
            ("auth", "def node AuthService:\n  port handle_login\n"),
            (
                "ui",
                "def node UI:\n  port login\ndef conn c := * -> *\nUI.login c AuthService.handle_login\n",
            ),
        ]);
        assert!(
            errs.iter()
                .any(|(c, m)| c == "E_NOT_VISIBLE" && m.contains("import auth")),
            "{errs:?}"
        );
    }

    #[test]
    fn selective_imports_gate_the_rest() {
        let errs = codes(&[
            ("messages", "def node LoginForm\ndef node AuthResponse\n"),
            (
                "ui",
                "import messages (LoginForm)\ndef node UI\ndef rel uses := * -> *\nUI uses LoginForm\nUI uses AuthResponse\n",
            ),
        ]);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].0, "E_NOT_VISIBLE");
        assert!(errs[0].1.contains("AuthResponse"));
    }

    #[test]
    fn unknown_modules_and_exports_are_reported() {
        let errs = codes(&[("main", "import nowhere\ndef node A\n")]);
        assert_eq!(errs[0].0, "E_UNKNOWN_MODULE");
        let errs = codes(&[
            ("m", "def node A\n"),
            ("main", "import m (B)\ndef node C\n"),
        ]);
        assert!(errs[0].1.contains("does not export"), "{errs:?}");
    }

    #[test]
    fn import_cycles_are_legal() {
        resolve_ok(&[
            (
                "a",
                "import b\ndef node A\ndef rel calls := * -> *\nA calls B\n",
            ),
            ("b", "import a\ndef node B\nB calls A\n"),
        ]);
    }

    #[test]
    fn one_definition_site_per_name() {
        let errs = codes(&[("a", "def node X\n"), ("b", "def node X\n")]);
        assert_eq!(errs[0].0, "E_REDECLARED");
        let errs = codes(&[("a", "def rel r := * -> *\ndef conn r := * -> *\n")]);
        assert_eq!(errs[0].0, "E_REDECLARED");
        // Preset names cannot be captured.
        let errs = codes(&[("a", "def node Service\n")]);
        assert_eq!(errs[0].0, "E_REDECLARED");
        assert!(errs[0].1.contains("preset"), "{errs:?}");
        let errs = codes(&[("a", "def rel type_of := * -> *\n")]);
        assert!(errs[0].1.contains("preset"), "{errs:?}");
    }

    #[test]
    fn opens_resolve_across_files_in_any_order() {
        // `zz_defs` sorts after `aa_open`: the open must wait for the def.
        let r = resolve_ok(&[
            (
                "aa_open",
                "import zz_defs\nopen Orders:\n  def node RefundHandler:\n    port handle\n",
            ),
            ("zz_defs", "def node Orders:\n  port refunds\n"),
        ]);
        assert!(r.nodes.contains_key("Orders.RefundHandler"));
        // And nested opens chain through semantic children defined elsewhere.
        let r = resolve_ok(&[
            (
                "deep",
                "import zz_defs\nimport mid\nopen Orders:\n  open RefundHandler:\n    def node Ledger\n",
            ),
            (
                "mid",
                "import zz_defs\nopen Orders:\n  def node RefundHandler:\n    port handle\n",
            ),
            ("zz_defs", "def node Orders\n"),
        ]);
        assert!(r.nodes.contains_key("Orders.RefundHandler.Ledger"));
    }

    #[test]
    fn unresolvable_opens_are_reported() {
        let errs = codes(&[("a", "open Ghost:\n  def node X\n")]);
        assert_eq!(errs[0].0, "E_UNKNOWN_NAME");
    }

    #[test]
    fn ports_must_be_declared() {
        let errs = codes(&[(
            "a",
            "def node A:\n  port out\ndef node B\ndef conn c := * -> *\nA.out c B.inbox\n",
        )]);
        assert_eq!(errs[0].0, "E_UNDECLARED_PORT");
        assert!(errs[0].1.contains("B"), "{errs:?}");
        // Ports on preset nodes cannot be used from source.
        let errs = codes(&[(
            "a",
            "def node A:\n  port out\ndef conn c := * -> *\nA.out c Service.inbox\n",
        )]);
        assert_eq!(errs[0].0, "E_UNDECLARED_PORT");
        assert!(errs[0].1.contains("preset"), "{errs:?}");
    }

    #[test]
    fn block_children_shadow_file_scope() {
        let r = resolve_ok(&[
            ("lib", "def node Cache:\n  port save\n"),
            (
                "svc",
                "import lib\ndef rel backs := * -> *\ndef node Svc:\n  def node Cache:\n    port save_local\n  Cache backs Svc.Handler\n  def node Handler:\n    port h\n",
            ),
        ]);
        // `Cache` inside the block is the child, not the imported root.
        assert!(r.edges.iter().any(|e| matches!(
            e,
            EdgeR::Rel { source, .. } if source == "Svc.Cache"
        )));
    }

    #[test]
    fn carrier_arguments_map_onto_lanes() {
        // A bare argument is ambiguous when both lanes carry.
        let errs = codes(&[(
            "a",
            "def node Q\ndef node R\ndef node A:\n  port p\ndef node B:\n  port q\ndef conn rpc := * ->(Data type_of *), <-(Data type_of *) *\nA.p rpc(Q) B.q\n",
        )]);
        assert!(errs[0].1.contains("tag"), "{errs:?}");
        // A pattern lane with no argument and no exact default is required.
        let errs = codes(&[(
            "a",
            "def node A:\n  port p\ndef node B:\n  port q\ndef conn send := * ->(Data type_of *) *\nA.p send B.q\n",
        )]);
        assert_eq!(errs[0].0, "E_CARRIER_REQUIRED");
    }

    #[test]
    fn apps_check_children_and_ports() {
        let r = resolve_ok(&[(
            "a",
            "def node Msg\ndef node Peer:\n  port out\ndef node Orders:\n  port events\n  def node OrderHandler:\n    port handle\n  events(Msg) = OrderHandler.handle\ndef conn send := * -> *\nPeer.out send Orders.events\n",
        )]);
        assert_eq!(r.apps.len(), 1);
        assert_eq!(r.apps[0].node, "Orders");
        assert_eq!(r.apps[0].inner_node, "OrderHandler");
        assert!(matches!(
            &r.apps[0].route,
            Some(PatternExpr::Exact { node }) if node == "Msg"
        ));
        let errs = codes(&[(
            "a",
            "def node Orders:\n  port events\n  events = Ghost.handle\n",
        )]);
        assert!(errs[0].1.contains("no child"), "{errs:?}");
    }
}
