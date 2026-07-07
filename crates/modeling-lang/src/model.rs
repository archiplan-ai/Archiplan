//! The model store: nodes with scopes and ports, edge types, views and edges.
//!
//! Names are handles; identity lives in ids. All references between stored
//! elements — edge ends, carriers, pattern anchors, delegations — are ids, so
//! renames are reference-safe by construction and deletion can compute the
//! exact referencing closure. All name resolution is absolute: paths descend
//! from the root, there is no ambient scope.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use serde_json::json;

use crate::error::{ErrorCode, LangError};
use crate::ids::{ConnId, EdgeId, NodeId, PortId, RelId, ViewId};
use crate::incidence::{IncidenceConfig, IncidenceReport, IncidenceRow, Invariant};
use crate::nkp::{NkpConfig, NkpReport};
use crate::result::Finding;
use crate::statement::Statement;

/// Which side of a directed connection type a port is fixed to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Side {
    Source,
    Target,
}

impl Side {
    pub fn describe(self) -> &'static str {
        match self {
            Side::Source => "source",
            Side::Target => "target",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub id: NodeId,
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: BTreeMap<String, NodeId>,
    pub ports: BTreeMap<String, PortId>,
}

/// A named attachment point on a node. Its connection type and, for directed
/// types, its side are fixed by its first use.
///
/// A **declared** port comes from its node's definition and exists before —
/// and beyond — any edge: `conn` is `None` until a first use fixes it, and
/// the port survives losing its last edge. A use-created port exists only
/// while attached.
#[derive(Clone, Debug)]
pub(crate) struct Port {
    pub id: PortId,
    pub node: NodeId,
    pub name: String,
    pub conn: Option<ConnId>,
    pub side: Option<Side>,
    pub declared: bool,
}

/// A resolved pattern. Anchors and relations are bound by id at declaration
/// time, so patterns survive renames.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Pattern {
    Any,
    Exact(NodeId),
    Classified { anchor: NodeId, rel: RelId },
}

#[derive(Clone, Debug)]
pub(crate) struct RelType {
    pub id: RelId,
    pub name: String,
    pub trans: bool,
    pub directed: bool,
    pub src: Pattern,
    pub dst: Pattern,
}

#[derive(Clone, Debug)]
pub(crate) struct ConnType {
    pub id: ConnId,
    pub name: String,
    pub directed: bool,
    pub src: Pattern,
    /// Forward-lane carried slot (source → target).
    pub carrier: Option<Pattern>,
    /// Reverse-lane carried slot (target → source); directed types only —
    /// direction means initiation, a reverse carrier is the response payload.
    pub rev_carrier: Option<Pattern>,
    pub dst: Pattern,
}

#[derive(Clone, Debug)]
pub(crate) struct ViewDef {
    pub id: ViewId,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum EdgePayload {
    Rel {
        rel: RelId,
        src: NodeId,
        dst: NodeId,
    },
    Conn {
        conn: ConnId,
        src_port: PortId,
        carrier: Option<NodeId>,
        rev_carrier: Option<NodeId>,
        dst_port: PortId,
    },
    App {
        outer: PortId,
        qualifier: Option<Pattern>,
        inner: PortId,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Edge {
    pub id: EdgeId,
    pub payload: EdgePayload,
    pub views: BTreeSet<ViewId>,
}

/// Which layer a node belongs to, per `requirements/modeling-lang/layers.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    /// A type: the node appears at least once on the left side of `type_of`.
    Epistemic,
    /// A term: a concrete structural element, never on the left of `type_of`.
    Epistatic,
}

/// A model: the store behind a [`crate::Workspace`].
///
/// Mutation goes through the statement API on `Workspace`; the model itself
/// exposes read access — [`Model::dump`], [`Model::check`], [`Model::layer_of`],
/// [`Model::nkp`].
#[derive(Clone, Debug)]
pub struct Model {
    next_id: u64,
    /// Ids below this bound belong to the preset loaded as the stdlib.
    stdlib_watermark: u64,
    preset_name: String,
    pub(crate) nodes: BTreeMap<NodeId, Node>,
    pub(crate) root: BTreeMap<String, NodeId>,
    pub(crate) ports: BTreeMap<PortId, Port>,
    pub(crate) rels: BTreeMap<RelId, RelType>,
    pub(crate) conns: BTreeMap<ConnId, ConnType>,
    pub(crate) views: BTreeMap<ViewId, ViewDef>,
    pub(crate) edges: BTreeMap<EdgeId, Edge>,
    pub(crate) rel_names: BTreeMap<String, RelId>,
    pub(crate) conn_names: BTreeMap<String, ConnId>,
    pub(crate) view_names: BTreeMap<String, ViewId>,
    pub(crate) type_of: RelId,
}

impl Model {
    /// A truly empty model: no stdlib yet. Only meaningful as the target of a
    /// preset load ([`crate::Workspace::with_preset`]), which must
    /// [`Model::seal_preset`] before the model is used.
    pub(crate) fn empty() -> Self {
        Model {
            next_id: 1,
            stdlib_watermark: 0,
            preset_name: String::new(),
            nodes: BTreeMap::new(),
            root: BTreeMap::new(),
            ports: BTreeMap::new(),
            rels: BTreeMap::new(),
            conns: BTreeMap::new(),
            views: BTreeMap::new(),
            edges: BTreeMap::new(),
            rel_names: BTreeMap::new(),
            conn_names: BTreeMap::new(),
            view_names: BTreeMap::new(),
            type_of: RelId(0),
        }
    }

    /// Whether the id belongs to the stdlib (preset) region.
    pub(crate) fn is_stdlib(&self, raw: u64) -> bool {
        raw < self.stdlib_watermark
    }

    /// The name of the preset loaded as this model's stdlib.
    pub fn preset_name(&self) -> &str {
        &self.preset_name
    }

    /// Seal the preset region: everything created so far becomes stdlib.
    /// Requires the preset to have defined the classifier relation `type_of`
    /// with the exact stdlib shape `rel trans type_of := * -> *` — the layer
    /// split and the `types` query filter key off it.
    pub(crate) fn seal_preset(&mut self, name: &str) -> Result<(), LangError> {
        let Some(&tid) = self.rel_names.get("type_of") else {
            return Err(LangError::new(
                ErrorCode::PresetInvalid,
                format!("preset `{name}` does not define the classifier relation `type_of`"),
            )
            .with_hint(json!({ "stmt": "define", "rel": "type_of", "trans": true,
                               "directed": true, "source": "*", "target": "*" })));
        };
        let rt = &self.rels[&tid];
        let conforms = rt.trans && rt.directed && rt.src == Pattern::Any && rt.dst == Pattern::Any;
        if !conforms {
            return Err(LangError::new(
                ErrorCode::PresetInvalid,
                format!(
                    "preset `{name}` defines `type_of` divergently; the classifier must conform to `rel trans type_of := * -> *`"
                ),
            )
            .with_expected(json!({ "stmt": "define", "rel": "type_of", "trans": true,
                                   "directed": true, "source": "*", "target": "*" }))
            .with_actual(self.rel_statement(rt).to_value()));
        }
        self.type_of = tid;
        self.preset_name = name.to_string();
        self.stdlib_watermark = self.next_id;
        Ok(())
    }

    pub(crate) fn alloc(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(crate) fn children(&self, scope: Option<NodeId>) -> &BTreeMap<String, NodeId> {
        match scope {
            Some(n) => &self.nodes[&n].children,
            None => &self.root,
        }
    }

    /// Absolute resolution: descend from the given base (root when `None`)
    /// through child scopes. The only resolution rule in the language.
    pub(crate) fn resolve_in(&self, base: Option<NodeId>, segs: &[String]) -> Option<NodeId> {
        let mut node = *self.children(base).get(&segs[0])?;
        for seg in &segs[1..] {
            node = *self.nodes[&node].children.get(seg)?;
        }
        Some(node)
    }

    /// The chain of nodes from the root down to and including `node`.
    pub(crate) fn scope_chain(&self, node: NodeId) -> Vec<NodeId> {
        let mut chain = vec![node];
        let mut cur = node;
        while let Some(p) = self.nodes[&cur].parent {
            chain.push(p);
            cur = p;
        }
        chain.reverse();
        chain
    }

    /// Absolute dot path of a node from the root.
    pub(crate) fn node_path(&self, node: NodeId) -> String {
        self.scope_chain(node)
            .iter()
            .map(|n| self.nodes[n].name.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Absolute `node.port` path of a port.
    pub(crate) fn port_path(&self, port: PortId) -> String {
        let p = &self.ports[&port];
        format!("{}.{}", self.node_path(p.node), p.name)
    }

    /// Names of the node's declared ports, in name order.
    pub(crate) fn declared_ports(&self, node: NodeId) -> Vec<String> {
        self.nodes[&node]
            .ports
            .values()
            .filter_map(|pid| {
                let p = &self.ports[pid];
                p.declared.then(|| p.name.clone())
            })
            .collect()
    }

    /// Whether `rel` holds from `from` to `to`, following the relation's
    /// direction and, for transitive relations, the virtual closure over
    /// declared edges.
    pub(crate) fn rel_holds(&self, rel: RelId, from: NodeId, to: NodeId) -> bool {
        let rt = &self.rels[&rel];
        let neighbors = |n: NodeId| {
            self.edges.values().filter_map(move |e| match &e.payload {
                EdgePayload::Rel { rel: r, src, dst } if *r == rel => {
                    if *src == n {
                        Some(*dst)
                    } else if !rt.directed && *dst == n {
                        Some(*src)
                    } else {
                        None
                    }
                }
                _ => None,
            })
        };
        if !rt.trans {
            return neighbors(from).any(|n| n == to);
        }
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([from]);
        while let Some(n) = queue.pop_front() {
            for next in neighbors(n) {
                if next == to {
                    return true;
                }
                if seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        false
    }

    pub(crate) fn matches(&self, pat: &Pattern, node: NodeId) -> bool {
        match pat {
            Pattern::Any => true,
            Pattern::Exact(n) => *n == node,
            Pattern::Classified { anchor, rel } => self.rel_holds(*rel, *anchor, node),
        }
    }

    pub(crate) fn conn_edges_on_port(&self, port: PortId) -> impl Iterator<Item = &Edge> {
        self.edges.values().filter(move |e| {
            matches!(&e.payload, EdgePayload::Conn { src_port, dst_port, .. }
                if *src_port == port || *dst_port == port)
        })
    }

    pub(crate) fn apps_on_outer_port(&self, port: PortId) -> impl Iterator<Item = &Edge> {
        self.edges
            .values()
            .filter(move |e| matches!(&e.payload, EdgePayload::App { outer, .. } if *outer == port))
    }

    pub(crate) fn port_attached(&self, port: PortId) -> bool {
        self.edges.values().any(|e| match &e.payload {
            EdgePayload::Conn {
                src_port, dst_port, ..
            } => *src_port == port || *dst_port == port,
            EdgePayload::App { outer, inner, .. } => *outer == port || *inner == port,
            EdgePayload::Rel { .. } => false,
        })
    }

    pub(crate) fn find_rel_edge(&self, rel: RelId, a: NodeId, b: NodeId) -> Option<EdgeId> {
        let directed = self.rels[&rel].directed;
        self.edges.values().find_map(|e| match &e.payload {
            EdgePayload::Rel { rel: r, src, dst } if *r == rel => {
                let hit = (*src == a && *dst == b) || (!directed && *src == b && *dst == a);
                hit.then_some(e.id)
            }
            _ => None,
        })
    }

    pub(crate) fn find_conn_edge(
        &self,
        conn: ConnId,
        a: PortId,
        carrier: Option<NodeId>,
        rev_carrier: Option<NodeId>,
        b: PortId,
    ) -> Option<EdgeId> {
        let directed = self.conns[&conn].directed;
        self.edges.values().find_map(|e| match &e.payload {
            EdgePayload::Conn {
                conn: c,
                src_port,
                carrier: cr,
                rev_carrier: rcr,
                dst_port,
            } if *c == conn => {
                let same = (*src_port == a && *dst_port == b)
                    || (!directed && *src_port == b && *dst_port == a);
                (same && *cr == carrier && *rcr == rev_carrier).then_some(e.id)
            }
            _ => None,
        })
    }

    pub(crate) fn find_app_edge(
        &self,
        outer: PortId,
        qualifier: &Option<Pattern>,
        inner: PortId,
    ) -> Option<EdgeId> {
        self.edges.values().find_map(|e| match &e.payload {
            EdgePayload::App {
                outer: o,
                qualifier: q,
                inner: i,
            } => (*o == outer && *i == inner && q == qualifier).then_some(e.id),
            _ => None,
        })
    }

    /// Whether an absolute dot path names a node of this model. The
    /// doc-source layer (`requirements/requirements.md`,
    /// `requirements/stressing.md`) validates its model references —
    /// `satisfied-by` elements, stressor affects — through this.
    pub fn has_node(&self, path: &str) -> bool {
        let segs: Vec<String> = path.split('.').map(str::to_string).collect();
        self.resolve_in(None, &segs).is_some()
    }

    /// Which layer a node belongs to; `None` if the path does not resolve.
    /// The path is absolute, from the root scope.
    pub fn layer_of(&self, path: &str) -> Option<Layer> {
        let segs: Vec<String> = path.split('.').map(str::to_string).collect();
        let node = self.resolve_in(None, &segs)?;
        let is_type = self.edges.values().any(|e| {
            matches!(&e.payload, EdgePayload::Rel { rel, src, .. }
                if *rel == self.type_of && *src == node)
        });
        Some(if is_type {
            Layer::Epistemic
        } else {
            Layer::Epistatic
        })
    }

    /// The whole model rendered as replayable statements, in creation order.
    /// Creations render as `define`s, which are idempotent, so replays are safe.
    pub fn dump(&self) -> Vec<Statement> {
        crate::query::dump(self)
    }

    /// Model-completeness findings.
    pub fn check(&self) -> Vec<Finding> {
        crate::query::check(self, None)
    }

    /// NKP landscape analysis over the slice picked by `config`
    /// (`requirements/scoring/nkp.md`).
    pub fn nkp(&self, config: &NkpConfig) -> Result<NkpReport, LangError> {
        crate::nkp::analyze(self, config)
    }

    /// The pressure surface of an absolute path
    /// (`requirements/scoring/incidence.md`): a term is the one-element
    /// surface holding itself; a type expands to the user terms its
    /// `type_of` closure classifies. `None` when the path resolves to
    /// nothing. Stressor affects and `satisfied-by` entries expand through
    /// this, each against the model their contract names.
    pub fn term_surface(&self, path: &str) -> Option<Vec<String>> {
        crate::incidence::term_surface(self, path)
    }

    /// Incidence analysis with this model as the frame: the stressor ×
    /// component matrix and its findings (`requirements/scoring/incidence.md`).
    /// Rows carry term paths already expanded against their session's own
    /// version ([`Model::term_surface`]).
    pub fn incidence(
        &self,
        rows: &[IncidenceRow],
        invariants: &[Invariant],
        config: &IncidenceConfig,
    ) -> IncidenceReport {
        crate::incidence::analyze(self, rows, invariants, config)
    }
}
