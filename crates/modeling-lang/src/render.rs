//! Rendering stored elements back into the statements that recreate them.
//!
//! All node references render as absolute paths from the root; creations
//! render as `define` statements, which are idempotent, so dumps replay
//! safely. [`Model::render_source`] renders the whole model as canonical
//! `.arch` source — the archived form of `requirements/versioning.md` — and
//! [`Model::scope_sources`] slices that render per root scope for
//! scope-version hashing.

use std::collections::BTreeMap;

use crate::ids::NodeId;
use crate::model::{ConnType, Edge, EdgePayload, Model, Pattern, RelType, ViewDef};
use crate::statement::{Definition, End, PatternExpr, Statement};

/// Canonical source fragments of one root scope, for scope-version hashing
/// (`requirements/versioning.md#versioning--scopes`).
#[derive(Clone, PartialEq, Debug)]
pub struct ScopeSource {
    /// Name of the root node (a top-level scope).
    pub path: String,
    /// Everything under the node: the defines of the node and its
    /// descendants, plus the edges and applications whose attachments all
    /// lie inside the subtree. Canonical order. Carried nodes are edge
    /// metadata and do not anchor membership.
    pub full: String,
    /// The node's define (carrying its declared ports) plus its boundary
    /// edges — edges with exactly one attachment inside the subtree.
    /// Canonical order.
    pub interface: String,
}

impl Model {
    pub(crate) fn pattern_expr(&self, p: &Pattern) -> PatternExpr {
        match p {
            Pattern::Any => PatternExpr::Any,
            Pattern::Exact(n) => PatternExpr::Exact {
                node: self.node_path(*n),
            },
            Pattern::Classified { anchor, rel } => PatternExpr::Classified {
                anchor: self.node_path(*anchor),
                rel: self.rels[rel].name.clone(),
            },
        }
    }

    pub(crate) fn node_statement(&self, node: NodeId) -> Statement {
        let declared = self.declared_ports(node);
        Statement::Define(Definition::Node {
            path: self.node_path(node),
            ports: (!declared.is_empty()).then_some(declared),
        })
    }

    pub(crate) fn view_statement(&self, v: &ViewDef) -> Statement {
        Statement::Define(Definition::View {
            name: v.name.clone(),
        })
    }

    pub(crate) fn rel_statement(&self, rt: &RelType) -> Statement {
        Statement::Define(Definition::Rel {
            name: rt.name.clone(),
            trans: rt.trans,
            directed: rt.directed,
            source: self.pattern_expr(&rt.src),
            target: self.pattern_expr(&rt.dst),
        })
    }

    pub(crate) fn conn_statement(&self, ct: &ConnType) -> Statement {
        Statement::Define(Definition::Conn {
            name: ct.name.clone(),
            directed: ct.directed,
            source: self.pattern_expr(&ct.src),
            carrier: ct.carrier.as_ref().map(|c| self.pattern_expr(c)),
            rev_carrier: ct.rev_carrier.as_ref().map(|c| self.pattern_expr(c)),
            target: self.pattern_expr(&ct.dst),
        })
    }

    fn view_names_of(&self, e: &Edge) -> Vec<String> {
        e.views.iter().map(|v| self.views[v].name.clone()).collect()
    }

    pub(crate) fn edge_statement(&self, e: &Edge) -> Statement {
        match &e.payload {
            EdgePayload::Rel { rel, src, dst } => Statement::RelEdge {
                rel: self.rels[rel].name.clone(),
                source: self.node_path(*src),
                target: self.node_path(*dst),
                views: self.view_names_of(e),
            },
            EdgePayload::Conn {
                conn,
                src_port,
                carrier,
                rev_carrier,
                dst_port,
            } => {
                let sp = &self.ports[src_port];
                let dp = &self.ports[dst_port];
                Statement::ConnEdge {
                    conn: self.conns[conn].name.clone(),
                    source: End {
                        node: self.node_path(sp.node),
                        port: sp.name.clone(),
                    },
                    carrier: carrier.map(|c| self.node_path(c)),
                    rev_carrier: rev_carrier.map(|c| self.node_path(c)),
                    target: End {
                        node: self.node_path(dp.node),
                        port: dp.name.clone(),
                    },
                    views: self.view_names_of(e),
                }
            }
            EdgePayload::App {
                outer,
                qualifier,
                inner,
            } => {
                let op = &self.ports[outer];
                let ip = &self.ports[inner];
                Statement::App {
                    node: self.node_path(op.node),
                    port: op.name.clone(),
                    route: qualifier.as_ref().map(|q| self.pattern_expr(q)),
                    inner: End {
                        node: self.nodes[&ip.node].name.clone(),
                        port: ip.name.clone(),
                    },
                }
            }
        }
    }

    /// The nodes an edge is attached to. Carried nodes are edge metadata:
    /// they neither anchor nor block subtree membership, mirroring the
    /// query filter contract.
    fn edge_attachments(&self, e: &Edge) -> Vec<NodeId> {
        match &e.payload {
            EdgePayload::Rel { src, dst, .. } => vec![*src, *dst],
            EdgePayload::Conn {
                src_port, dst_port, ..
            } => vec![self.ports[src_port].node, self.ports[dst_port].node],
            EdgePayload::App { outer, inner, .. } => {
                vec![self.ports[outer].node, self.ports[inner].node]
            }
        }
    }

    /// The whole model rendered as canonical `.arch` source: the statements
    /// of [`Model::dump`], one per line, in the surface syntax. This is the
    /// canonical form of `requirements/versioning.md`: compiling the result
    /// as a single module against the same preset recreates the identical
    /// model, and re-rendering that model reproduces the text byte for byte.
    pub fn render_source(&self) -> String {
        let mut out = String::new();
        for s in self.dump() {
            out.push_str(&s.pseudo());
            out.push('\n');
        }
        out
    }

    /// The canonical render sliced per root scope, in root-name order.
    /// Stdlib (preset) roots are omitted; edges that touch them (such as
    /// `type_of` classifications) still appear as boundary edges of the
    /// user scopes they attach to.
    pub fn scope_sources(&self) -> Vec<ScopeSource> {
        let root_of = |mut n: NodeId| {
            while let Some(p) = self.nodes[&n].parent {
                n = p;
            }
            n
        };
        // Per-root buckets of (creation id, statement), split full/boundary.
        let mut full: BTreeMap<NodeId, Vec<(u64, Statement)>> = BTreeMap::new();
        let mut boundary: BTreeMap<NodeId, Vec<(u64, Statement)>> = BTreeMap::new();
        let user_roots: Vec<NodeId> = self
            .root
            .values()
            .copied()
            .filter(|r| !self.is_stdlib(r.raw()))
            .collect();
        for r in &user_roots {
            full.insert(*r, Vec::new());
            boundary.insert(*r, Vec::new());
        }
        for n in self.nodes.values() {
            if self.is_stdlib(n.id.raw()) {
                continue;
            }
            if let Some(items) = full.get_mut(&root_of(n.id)) {
                items.push((n.id.raw(), self.node_statement(n.id)));
            }
        }
        for e in self.edges.values() {
            if self.is_stdlib(e.id.raw()) {
                continue;
            }
            let mut roots: Vec<NodeId> =
                self.edge_attachments(e).into_iter().map(root_of).collect();
            roots.dedup();
            let inside_one = roots.len() == 1;
            for r in roots {
                let bucket = if inside_one { &mut full } else { &mut boundary };
                if let Some(items) = bucket.get_mut(&r) {
                    items.push((e.id.raw(), self.edge_statement(e)));
                }
            }
        }
        let text = |mut items: Vec<(u64, Statement)>| {
            items.sort_by_key(|(id, _)| *id);
            let mut out = String::new();
            for (_, s) in items {
                out.push_str(&s.pseudo());
                out.push('\n');
            }
            out
        };
        self.root
            .iter()
            .filter(|(_, r)| !self.is_stdlib(r.raw()))
            .map(|(name, r)| {
                let mut interface = vec![(r.raw(), self.node_statement(*r))];
                interface.extend(boundary.remove(r).expect("bucket per user root"));
                ScopeSource {
                    path: name.clone(),
                    full: text(full.remove(r).expect("bucket per user root")),
                    interface: text(interface),
                }
            })
            .collect()
    }
}
