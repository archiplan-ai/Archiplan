//! NKP landscape analysis: slicing the model into its epistatic coupling
//! graph and reading where it sits on the order ↔ chaos spectrum
//! (`archi/requirements/scoring/the-landscape-is-a-slice.md`, background in `kb/nkp.md`).
//!
//! The slice is configuration-driven rather than hardwired to an ontology:
//!
//! - stdlib (preset) elements never participate — they are scaffolding;
//! - [`ExcludePattern`]s disqualify nodes: `_ type_of *` (anything on the
//!   left of `type_of` — the epistemic layer), `Data type_of _` (the
//!   transitive instances of `Data`);
//! - an [`NkpScope`] picks granularity: one level, or recursive top-to-bottom
//!   where delegation applications *fold* — a node realizing its parent's
//!   port counts as part of the parent, and its couplings re-attach there;
//! - connection carriers are metadata, not attachments: data carried on a
//!   connection never blocks the edge; only data as an *endpoint* drops it.
//!
//! Implemented artifacts: K/P metrics with regime classification, the binary
//! dependency matrix, coupling hotspots, and neutral corridors. Adaptive-walk
//! simulation and spectral cluster decomposition are not implemented yet;
//! the report's `notes` say so.
//!
//! Determinism: every collection is ordered (`BTreeMap`/`BTreeSet`), node
//! order is creation (id) order, so identical models yield identical reports.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

use crate::error::{ErrorCode, LangError};
use crate::ids::{NodeId, RelId};
use crate::model::{EdgePayload, Model};

// ---- configuration ---------------------------------------------------------

/// One slot of an [`ExcludePattern`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Slot {
    /// `_` — the nodes filling this slot are excluded.
    Excluded,
    /// `*` — any node.
    Any,
    /// An absolute node path, matched exactly.
    Node(String),
}

impl Slot {
    fn parse(token: &str) -> Slot {
        match token {
            "_" => Slot::Excluded,
            "*" => Slot::Any,
            path => Slot::Node(path.to_string()),
        }
    }

    fn pseudo(&self) -> String {
        match self {
            Slot::Excluded => "_".to_string(),
            Slot::Any => "*".to_string(),
            Slot::Node(p) => p.clone(),
        }
    }
}

/// An edge-shaped node-exclusion pattern over a named relation, written
/// `<source> <rel> <target>` with exactly one slot being `_`. A node is
/// excluded when it can fill the `_` slot such that the relation holds —
/// following the relation's transitive closure when it is `trans`:
///
/// - `_ type_of *` — every node appearing as a `type_of` source: the
///   epistemic layer (types), exactly the set [`Model::layer_of`] calls
///   [`crate::Layer::Epistemic`].
/// - `Data type_of _` — every node `type_of` reaches from `Data`: its
///   transitive instances.
///
/// A pattern whose relation or anchor does not resolve matches nothing and
/// surfaces as an `UNKNOWN_EXCLUDE_REF` warning in the report.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExcludePattern {
    source: Slot,
    rel: String,
    target: Slot,
}

impl ExcludePattern {
    /// Build a pattern; exactly one slot must be [`Slot::Excluded`].
    pub fn new(source: Slot, rel: impl Into<String>, target: Slot) -> Result<Self, LangError> {
        let underscores = [&source, &target]
            .into_iter()
            .filter(|s| **s == Slot::Excluded)
            .count();
        if underscores != 1 {
            return Err(LangError::new(
                ErrorCode::Parse,
                "an exclusion pattern names exactly one `_` slot",
            ));
        }
        Ok(ExcludePattern {
            source,
            rel: rel.into(),
            target,
        })
    }

    /// Parse the compact form `<source> <rel> <target>`, e.g. `_ type_of *`
    /// or `Data type_of _`.
    pub fn parse(s: &str) -> Result<Self, LangError> {
        let tokens: Vec<&str> = s.split_whitespace().collect();
        let [src, rel, dst] = tokens.as_slice() else {
            return Err(LangError::new(
                ErrorCode::Parse,
                format!("an exclusion pattern is `<source> <rel> <target>`, got `{s}`"),
            ));
        };
        Self::new(Slot::parse(src), *rel, Slot::parse(dst))
    }

    /// The pattern in its compact form.
    pub fn pseudo(&self) -> String {
        format!(
            "{} {} {}",
            self.source.pseudo(),
            self.rel,
            self.target.pseudo()
        )
    }
}

impl std::fmt::Display for ExcludePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.pseudo())
    }
}

/// Scope granularity of an NKP analysis.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum NkpScope {
    /// Every scope, top to bottom. Delegation applications fold: a node
    /// realizing its parent's port merges into the parent, and its couplings
    /// re-attach to the merged unit.
    #[default]
    Recursive,
    /// Top-level nodes only.
    TopLevel,
    /// The direct children of one node (an absolute path) — the components
    /// of that area. No folding: a delegated child *is* a component here.
    Children(String),
}

/// How per-node neutrality P_i is assigned.
#[derive(Clone, PartialEq, Debug)]
pub enum Neutrality {
    /// `P_i = 1 − K_i / K_max`: sparsely coupled nodes are highly neutral.
    /// When no node has any coupling, every P_i is 1.
    DegreeDerived,
    /// One global P for every node.
    Uniform(f64),
}

/// NKP analysis configuration. `Default` matches the default ontology
/// preset: exclude `_ type_of *` and `Data type_of _`, recursive scope, all
/// edge types, degree-derived neutrality, τ_P = 0.6, τ_B = 0.3.
#[derive(Clone, Debug)]
pub struct NkpConfig {
    /// Node-exclusion patterns; a node matching any pattern's `_` slot is
    /// out of the landscape.
    pub exclude: Vec<ExcludePattern>,
    /// Scope granularity.
    pub scope: NkpScope,
    /// When set, only edges of these rel/conn type names count as coupling.
    pub only_edge_types: Option<Vec<String>>,
    /// Neutrality strategy.
    pub neutrality: Neutrality,
    /// Neutrality threshold for corridor membership.
    pub tau_p: f64,
    /// Boundary-exposure threshold for a SAFE corridor.
    pub tau_b: f64,
    /// How heavily coupling weighs on degree-derived neutrality — the
    /// operator's trade-off situates the read (`Tradeoffs`,
    /// `archi/requirements/tradeoff-configuration/priorities-weight-the-read.md`).
    /// `1.0` is unweighted: the read is byte-identical to an unsituated one.
    pub coupling_emphasis: f64,
}

impl Default for NkpConfig {
    fn default() -> Self {
        NkpConfig {
            exclude: vec![
                ExcludePattern::parse("_ type_of *").expect("default pattern is valid"),
                ExcludePattern::parse("Data type_of _").expect("default pattern is valid"),
            ],
            scope: NkpScope::Recursive,
            only_edge_types: None,
            neutrality: Neutrality::DegreeDerived,
            tau_p: 0.6,
            tau_b: 0.3,
            coupling_emphasis: 1.0,
        }
    }
}

// ---- report ----------------------------------------------------------------

/// Regime classification by mean connectivity K̄.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Regime {
    /// K̄ < 1: changes rarely propagate.
    #[serde(rename = "ORDERED")]
    Ordered,
    /// 1 ≤ K̄ ≤ 3: the evolvable edge of chaos — the target.
    #[serde(rename = "CRITICAL")]
    Critical,
    /// K̄ > 3: every change ripples.
    #[serde(rename = "CHAOTIC")]
    Chaotic,
}

impl Regime {
    /// The report label — the same string the serde rename emits.
    pub fn describe(self) -> &'static str {
        match self {
            Regime::Ordered => "ORDERED",
            Regime::Critical => "CRITICAL",
            Regime::Chaotic => "CHAOTIC",
        }
    }
}

/// What the slice looked like.
#[derive(Clone, Debug, Serialize)]
pub struct NkpScopeInfo {
    /// The preset whose stdlib was excluded from the slice.
    pub preset: String,
    /// Scope mode, rendered: `recursive`, `top-level` or `children of X`.
    pub scope: String,
    /// The exclusion patterns, in compact form.
    pub exclude: Vec<String>,
    /// How many in-scope nodes the patterns excluded.
    pub excluded_nodes: usize,
    /// Folded delegations: representative → the nodes it absorbed.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub folded: BTreeMap<String, Vec<String>>,
    /// N — nodes in the landscape.
    pub node_count: usize,
    /// Coupled ordered pairs — the 1-entries of the dependency matrix.
    pub edge_count: usize,
    /// Neutrality strategy, rendered.
    pub neutrality_strategy: String,
}

/// The landscape metrics.
#[derive(Clone, Debug, Serialize)]
pub struct NkpMetrics {
    /// Mean in-degree K̄ over distinct coupling sources.
    #[serde(rename = "K_bar")]
    pub k_bar: f64,
    /// Standard deviation of K_i.
    #[serde(rename = "K_std")]
    pub k_std: f64,
    /// Mean neutrality P̄.
    #[serde(rename = "P_bar")]
    pub p_bar: f64,
    /// ORDERED / CRITICAL / CHAOTIC.
    pub regime: Regime,
    /// log₂ of the analytical local-optima estimate 2^N / (K̄ + 1).
    pub local_optima_log2: f64,
    /// Weinberger correlation length ξ ≈ 1 / K̄; absent when K̄ = 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_length: Option<f64>,
}

/// The binary dependency matrix: `rows[i][j] = 1` iff node `j` couples into
/// node `i`.
#[derive(Clone, Debug, Serialize)]
pub struct NkpMatrix {
    /// Row/column order: node paths in creation order.
    pub nodes: Vec<String>,
    /// N×N adjacency rows.
    pub rows: Vec<Vec<u8>>,
}

/// A node whose in-degree exceeds K̄ + σ_K: a coupling hotspot, the
/// highest-risk refactoring target.
#[derive(Clone, Debug, Serialize)]
pub struct Hotspot {
    /// Node path.
    pub node: String,
    /// Its in-degree.
    pub k_in: usize,
}

/// Corridor safety label.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum CorridorLabel {
    /// Boundary exposure below τ_B: a safe refactoring zone.
    #[serde(rename = "SAFE_CORRIDOR")]
    Safe,
    /// Neutral inside but too exposed at the boundary.
    #[serde(rename = "PARTIALLY_NEUTRAL")]
    PartiallyNeutral,
}

impl CorridorLabel {
    /// The report label — the same string the serde rename emits.
    pub fn describe(self) -> &'static str {
        match self {
            CorridorLabel::Safe => "SAFE_CORRIDOR",
            CorridorLabel::PartiallyNeutral => "PARTIALLY_NEUTRAL",
        }
    }
}

/// Suggested refactoring action for a corridor.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum CorridorAction {
    /// Low boundary exposure and low internal coupling: wrap it up.
    #[serde(rename = "ENCAPSULATE")]
    Encapsulate,
    /// Large and internally cohesive: extract as a module.
    #[serde(rename = "EXTRACT_MODULE")]
    ExtractModule,
    /// Moderate boundary exposure: shrink the interface first.
    #[serde(rename = "SIMPLIFY_INTERFACE")]
    SimplifyInterface,
}

impl CorridorAction {
    /// The report label — the same string the serde rename emits.
    pub fn describe(self) -> &'static str {
        match self {
            CorridorAction::Encapsulate => "ENCAPSULATE",
            CorridorAction::ExtractModule => "EXTRACT_MODULE",
            CorridorAction::SimplifyInterface => "SIMPLIFY_INTERFACE",
        }
    }
}

/// A maximal connected set of neutral nodes (P_i ≥ τ_P).
#[derive(Clone, Debug, Serialize)]
pub struct NkpCorridor {
    /// Stable id within the report: `C0`, `C1`, ... in node-creation order.
    pub id: String,
    /// Node paths.
    pub nodes: Vec<String>,
    /// Mean in-degree counting only intra-corridor coupling.
    #[serde(rename = "K_bar_internal")]
    pub k_bar_internal: f64,
    /// Boundary adjacencies to non-neutral nodes, divided by corridor size.
    pub boundary_exposure: f64,
    /// SAFE_CORRIDOR or PARTIALLY_NEUTRAL.
    pub label: CorridorLabel,
    /// Suggested refactoring action, when one applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<CorridorAction>,
    /// 1 − boundary_exposure, clamped to [0, 1].
    pub confidence: f64,
}

/// A non-fatal analysis warning.
#[derive(Clone, Debug, Serialize)]
pub struct NkpWarning {
    /// Stable code: `SMALL_N`, `DISCONNECTED`, `NO_NEUTRAL_NODES`,
    /// `FULLY_CHAOTIC` or `UNKNOWN_EXCLUDE_REF`.
    pub code: &'static str,
    /// Human-readable one-liner.
    pub message: String,
}

/// The NKP analysis report — self-contained and renderable independently of
/// the graph store.
#[derive(Clone, Debug, Serialize)]
pub struct NkpReport {
    /// The slice this analysis ran on.
    pub scope: NkpScopeInfo,
    /// Landscape metrics.
    pub metrics: NkpMetrics,
    /// The dependency matrix.
    pub matrix: NkpMatrix,
    /// Coupling hotspots, highest in-degree first.
    pub hotspots: Vec<Hotspot>,
    /// Neutral corridors.
    pub neutral_corridors: Vec<NkpCorridor>,
    /// Analysis warnings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<NkpWarning>,
    /// Stages this implementation does not run yet.
    pub notes: Vec<String>,
}

// ---- analysis --------------------------------------------------------------

fn resolve_path(model: &Model, path: &str) -> Option<NodeId> {
    let segs: Vec<String> = path.split('.').map(str::to_string).collect();
    model.resolve_in(None, &segs)
}

/// The nodes on one side of a relation's declared edges; for a non-directed
/// relation both sides qualify. For a `trans` relation the declared sides
/// already cover the closure: any transitive source starts with a declared
/// edge of its own.
pub(crate) fn ends_of(model: &Model, rel: RelId, source_side: bool) -> BTreeSet<NodeId> {
    let directed = model.rels[&rel].directed;
    let mut out = BTreeSet::new();
    for e in model.edges.values() {
        if let EdgePayload::Rel { rel: r, src, dst } = &e.payload
            && *r == rel
        {
            if source_side || !directed {
                out.insert(*src);
            }
            if !source_side || !directed {
                out.insert(*dst);
            }
        }
    }
    out
}

/// Everything the relation reaches from `start` (or reaches `start`, when
/// `forward` is false): the transitive closure for `trans` relations, one
/// step otherwise. `start` itself is included only if a cycle reaches it.
pub(crate) fn reach(model: &Model, rel: RelId, start: NodeId, forward: bool) -> BTreeSet<NodeId> {
    let rt = &model.rels[&rel];
    let neighbors = |n: NodeId| -> Vec<NodeId> {
        model
            .edges
            .values()
            .filter_map(|e| match &e.payload {
                EdgePayload::Rel { rel: r, src, dst } if *r == rel => {
                    if !rt.directed {
                        if *src == n {
                            Some(*dst)
                        } else if *dst == n {
                            Some(*src)
                        } else {
                            None
                        }
                    } else if forward {
                        (*src == n).then_some(*dst)
                    } else {
                        (*dst == n).then_some(*src)
                    }
                }
                _ => None,
            })
            .collect()
    };
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([start]);
    while let Some(n) = queue.pop_front() {
        for m in neighbors(n) {
            if seen.insert(m) && rt.trans {
                queue.push_back(m);
            }
        }
        if !rt.trans {
            break;
        }
    }
    seen
}

/// Evaluate one exclusion pattern to the set of nodes filling its `_` slot;
/// `Err` names the reference that does not resolve.
fn eval_exclude(model: &Model, pat: &ExcludePattern) -> Result<BTreeSet<NodeId>, String> {
    let Some(&rel) = model.rel_names.get(&pat.rel) else {
        return Err(format!("rel `{}`", pat.rel));
    };
    let resolve = |p: &str| resolve_path(model, p).ok_or_else(|| format!("node `{p}`"));
    Ok(match (&pat.source, &pat.target) {
        (Slot::Excluded, Slot::Any) => ends_of(model, rel, true),
        (Slot::Any, Slot::Excluded) => ends_of(model, rel, false),
        (Slot::Excluded, Slot::Node(p)) => reach(model, rel, resolve(p)?, false),
        (Slot::Node(p), Slot::Excluded) => reach(model, rel, resolve(p)?, true),
        // The constructor guarantees exactly one `_` slot.
        _ => unreachable!("an ExcludePattern has exactly one `_` slot"),
    })
}

/// Weakly-connected components of `nodes` under `adj`, in node-creation
/// order. `adj` may cover a superset; membership is gated by `nodes`.
fn components(
    nodes: &BTreeSet<NodeId>,
    adj: &BTreeMap<NodeId, BTreeSet<NodeId>>,
) -> Vec<BTreeSet<NodeId>> {
    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut out = Vec::new();
    for &start in nodes {
        if !seen.insert(start) {
            continue;
        }
        let mut comp = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(x) = queue.pop_front() {
            for &y in adj.get(&x).into_iter().flatten() {
                if nodes.contains(&y) && seen.insert(y) {
                    comp.insert(y);
                    queue.push_back(y);
                }
            }
        }
        out.push(comp);
    }
    out
}

pub(crate) fn analyze(model: &Model, config: &NkpConfig) -> Result<NkpReport, LangError> {
    let mut warnings: Vec<NkpWarning> = Vec::new();

    // Scope: candidate nodes. Stdlib (preset) nodes never participate.
    let user = |n: &NodeId| !model.is_stdlib(n.raw());
    let candidates: BTreeSet<NodeId> = match &config.scope {
        NkpScope::Recursive => model.nodes.keys().filter(|n| user(n)).copied().collect(),
        NkpScope::TopLevel => model.root.values().filter(|n| user(n)).copied().collect(),
        NkpScope::Children(path) => {
            let n =
                resolve_path(model, path).ok_or_else(|| {
                    LangError::new(ErrorCode::UnknownName, format!("unknown node `{path}`"))
                        .with_ref("node", path.clone(), None)
                })?;
            model.nodes[&n]
                .children
                .values()
                .filter(|c| user(c))
                .copied()
                .collect()
        }
    };

    // Folding: in recursive mode, applications merge their inner node into
    // the delegating node. The parent chain is acyclic by construction, so
    // following it terminates.
    let mut fold_parent: BTreeMap<NodeId, NodeId> = BTreeMap::new();
    if config.scope == NkpScope::Recursive {
        for e in model.edges.values() {
            if let EdgePayload::App { outer, inner, .. } = &e.payload {
                let o = model.ports[outer].node;
                let i = model.ports[inner].node;
                if i != o {
                    fold_parent.insert(i, o);
                }
            }
        }
    }
    let rep = |mut n: NodeId| -> NodeId {
        while let Some(&p) = fold_parent.get(&n) {
            n = p;
        }
        n
    };

    // Exclusion patterns.
    let mut excluded: BTreeSet<NodeId> = BTreeSet::new();
    for pat in &config.exclude {
        match eval_exclude(model, pat) {
            Ok(set) => excluded.extend(set),
            Err(missing) => warnings.push(NkpWarning {
                code: "UNKNOWN_EXCLUDE_REF",
                message: format!(
                    "exclusion pattern `{}` matches nothing: unknown {missing}",
                    pat.pseudo()
                ),
            }),
        }
    }

    // Units: representatives of candidates. A merged unit stands or falls
    // with its representative: excluded rep → the whole unit is out; a rep
    // folded into stdlib scaffolding is out entirely.
    let mut members: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for &c in &candidates {
        members.entry(rep(c)).or_default().insert(c);
    }
    let mut excluded_nodes = 0usize;
    let mut units: BTreeSet<NodeId> = BTreeSet::new();
    for (&r, ms) in &members {
        if model.is_stdlib(r.raw()) {
            continue;
        }
        if excluded.contains(&r) {
            excluded_nodes += ms.len();
            continue;
        }
        units.insert(r);
    }

    // Coupling edges: relations and connections whose endpoint units both
    // survive. Applications are folded, never coupling. The carrier of a
    // ternary connection is metadata and never blocks the edge.
    let mut in_nbrs: BTreeMap<NodeId, BTreeSet<NodeId>> =
        units.iter().map(|&u| (u, BTreeSet::new())).collect();
    for e in model.edges.values() {
        let (ends, type_name, directed) = match &e.payload {
            EdgePayload::Rel { rel, src, dst } => {
                let rt = &model.rels[rel];
                ([*src, *dst], rt.name.as_str(), rt.directed)
            }
            EdgePayload::Conn {
                conn,
                src_port,
                dst_port,
                ..
            } => {
                let ct = &model.conns[conn];
                (
                    [model.ports[src_port].node, model.ports[dst_port].node],
                    ct.name.as_str(),
                    ct.directed,
                )
            }
            EdgePayload::App { .. } => continue,
        };
        if let Some(only) = &config.only_edge_types
            && !only.iter().any(|t| t == type_name)
        {
            continue;
        }
        let (a, b) = (rep(ends[0]), rep(ends[1]));
        if a == b || !units.contains(&a) || !units.contains(&b) {
            continue;
        }
        in_nbrs.get_mut(&b).expect("unit exists").insert(a);
        if !directed {
            in_nbrs.get_mut(&a).expect("unit exists").insert(b);
        }
    }

    // Metrics.
    let n = units.len();
    let nf = n as f64;
    let k: Vec<usize> = units.iter().map(|u| in_nbrs[u].len()).collect();
    let edge_count: usize = k.iter().sum();
    let k_bar = if n == 0 { 0.0 } else { edge_count as f64 / nf };
    let k_std = if n == 0 {
        0.0
    } else {
        (k.iter().map(|&ki| (ki as f64 - k_bar).powi(2)).sum::<f64>() / nf).sqrt()
    };
    let regime = if k_bar < 1.0 {
        Regime::Ordered
    } else if k_bar <= 3.0 {
        Regime::Critical
    } else {
        Regime::Chaotic
    };
    let k_max = k.iter().copied().max().unwrap_or(0);
    let p: Vec<f64> = match config.neutrality {
        Neutrality::DegreeDerived => k
            .iter()
            .map(|&ki| {
                if k_max == 0 {
                    1.0
                } else {
                    // Coupling weighs on neutrality by the operator's emphasis;
                    // at 1.0 this is exactly `1 − K_i/K_max` (`1.0 * x == x`).
                    (1.0 - config.coupling_emphasis * (ki as f64 / k_max as f64)).max(0.0)
                }
            })
            .collect(),
        Neutrality::Uniform(global_p) => vec![global_p; n],
    };
    let p_bar = if n == 0 {
        0.0
    } else {
        p.iter().sum::<f64>() / nf
    };
    let local_optima_log2 = nf - (k_bar + 1.0).log2();
    let correlation_length = (k_bar > 0.0).then(|| 1.0 / k_bar);

    // Dependency matrix, rows/cols in unit (creation) order.
    let index: BTreeMap<NodeId, usize> = units.iter().enumerate().map(|(i, &u)| (u, i)).collect();
    let rows: Vec<Vec<u8>> = units
        .iter()
        .map(|u| {
            let mut row = vec![0u8; n];
            for s in &in_nbrs[u] {
                row[index[s]] = 1;
            }
            row
        })
        .collect();

    // Hotspots: K_i above K̄ + σ_K.
    let mut hotspots: Vec<Hotspot> = units
        .iter()
        .zip(&k)
        .filter(|&(_, &ki)| (ki as f64) > k_bar + k_std)
        .map(|(&u, &ki)| Hotspot {
            node: model.node_path(u),
            k_in: ki,
        })
        .collect();
    hotspots.sort_by(|a, b| b.k_in.cmp(&a.k_in).then_with(|| a.node.cmp(&b.node)));

    // Undirected adjacency over units, for connectivity and corridors.
    let mut adj: BTreeMap<NodeId, BTreeSet<NodeId>> =
        units.iter().map(|&u| (u, BTreeSet::new())).collect();
    for (&u, nbrs) in &in_nbrs {
        for &v in nbrs {
            adj.get_mut(&u).expect("unit exists").insert(v);
            adj.get_mut(&v).expect("unit exists").insert(u);
        }
    }

    // Neutral corridors.
    let neutral: BTreeSet<NodeId> = units
        .iter()
        .zip(&p)
        .filter(|&(_, &pi)| pi >= config.tau_p)
        .map(|(&u, _)| u)
        .collect();
    let mut corridors = Vec::new();
    for (i, comp) in components(&neutral, &adj).iter().enumerate() {
        let size = comp.len() as f64;
        let internal: usize = comp
            .iter()
            .map(|u| in_nbrs[u].iter().filter(|s| comp.contains(s)).count())
            .sum();
        let k_bar_internal = internal as f64 / size;
        let boundary: usize = comp
            .iter()
            .map(|u| adj[u].iter().filter(|v| !neutral.contains(v)).count())
            .sum();
        let boundary_exposure = boundary as f64 / size;
        let label = if boundary_exposure < config.tau_b {
            CorridorLabel::Safe
        } else {
            CorridorLabel::PartiallyNeutral
        };
        let action = match label {
            CorridorLabel::Safe => Some(if k_bar_internal >= 1.0 && size > nf / 4.0 {
                CorridorAction::ExtractModule
            } else {
                CorridorAction::Encapsulate
            }),
            CorridorLabel::PartiallyNeutral => (boundary_exposure < 2.0 * config.tau_b)
                .then_some(CorridorAction::SimplifyInterface),
        };
        corridors.push(NkpCorridor {
            id: format!("C{i}"),
            nodes: comp.iter().map(|&u| model.node_path(u)).collect(),
            k_bar_internal,
            boundary_exposure,
            label,
            action,
            confidence: (1.0 - boundary_exposure).clamp(0.0, 1.0),
        });
    }

    // Warnings.
    if n < 4 {
        warnings.push(NkpWarning {
            code: "SMALL_N",
            message: format!("N = {n} is too small for reliable landscape statistics"),
        });
    }
    if components(&units, &adj).len() > 1 {
        warnings.push(NkpWarning {
            code: "DISCONNECTED",
            message: "the coupling graph is disconnected; metrics aggregate over all components"
                .to_string(),
        });
    }
    if n > 0 && neutral.is_empty() {
        warnings.push(NkpWarning {
            code: "NO_NEUTRAL_NODES",
            message: format!(
                "no node passes P_i ≥ {}; lower --tau-p or switch the neutrality strategy",
                config.tau_p
            ),
        });
    }
    if regime == Regime::Chaotic {
        warnings.push(NkpWarning {
            code: "FULLY_CHAOTIC",
            message: "K̄ > 3: the landscape is chaotic; decompose hotspots before refactoring"
                .to_string(),
        });
    }

    // Scope info.
    let folded: BTreeMap<String, Vec<String>> = members
        .iter()
        .filter(|(r, ms)| units.contains(r) && ms.len() > 1)
        .map(|(&r, ms)| {
            (
                model.node_path(r),
                ms.iter()
                    .filter(|&&m| m != r)
                    .map(|&m| model.node_path(m))
                    .collect(),
            )
        })
        .collect();
    let scope = NkpScopeInfo {
        preset: model.preset_name().to_string(),
        scope: match &config.scope {
            NkpScope::Recursive => "recursive".to_string(),
            NkpScope::TopLevel => "top-level".to_string(),
            NkpScope::Children(p) => format!("children of {p}"),
        },
        exclude: config.exclude.iter().map(ExcludePattern::pseudo).collect(),
        excluded_nodes,
        folded,
        node_count: n,
        edge_count,
        neutrality_strategy: match config.neutrality {
            Neutrality::DegreeDerived => "degree_derived".to_string(),
            Neutrality::Uniform(global_p) => format!("uniform_p({global_p})"),
        },
    };

    Ok(NkpReport {
        scope,
        metrics: NkpMetrics {
            k_bar,
            k_std,
            p_bar,
            regime,
            local_optima_log2,
            correlation_length,
        },
        matrix: NkpMatrix {
            nodes: units.iter().map(|&u| model.node_path(u)).collect(),
            rows,
        },
        hotspots,
        neutral_corridors: corridors,
        warnings,
        notes: vec![
            "adaptive-walk simulation is not implemented; walk statistics are omitted".to_string(),
            "spectral cluster decomposition is not implemented; the matrix is unclustered"
                .to_string(),
        ],
    })
}
