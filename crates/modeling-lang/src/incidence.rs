//! Incidence analysis: the stressor × component matrix
//! (`archi/requirements/scoring/the-matrix-joins-stress-to-structure.md`). Rows are stressors, columns are the
//! epistatic terms of one **frame** model, and a cell is 1 iff the term lies
//! on the stressor's pressure surface. The matrix surfaces couplings the
//! declared edges alone don't show.
//!
//! The caller owns everything session-shaped: it expands each stressor's
//! affects against *its own* pinned version ([`Model::term_surface`]) and
//! hands the resulting term paths in as [`IncidenceRow`]s; this module joins
//! them against the frame. Affected paths that are not terms of the frame —
//! drift between versions — are reported as `dropped`, never silently lost.
//!
//! Findings, by kind:
//!
//! - **hyperliminal coupling** — two columns whose stressor sets are
//!   near-identical (Jaccard ≥ τ_J over ≥ 2 shared stressors) with *no*
//!   declared path between them within `depth` hops: a hidden dependency.
//! - **merge candidate** — the same response similarity *over* a declared
//!   path: two nodes that may really be one, or share an extractable concern.
//! - **stress hotspot** — a column pressed by ≥ 2 stressors making up a
//!   τ_D fraction of the scope: disproportionate pressure.
//! - **density alert** — the matrix denser than τ_K over ≥ 3 stressors:
//!   stress is landing everywhere at once.
//! - **boundary-crossing stressor** — a row pressing ≥ 2 terms and far
//!   more of the frame than typical (w > w̄ + σ; alert past 2σ): likely
//!   crossing a boundary the architecture should make explicit.
//! - **compound vulnerability** — two *surviving* stressors, neither of
//!   which alone covers an invariant, whose union of affected terms does:
//!   individually answered, jointly a broken initial promise.
//! - **under-stressed** — a zero column: no stressor has touched the term.
//!
//! Determinism: every collection is ordered, columns come in node-creation
//! order, rows in caller order, so identical inputs yield identical reports.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::ids::NodeId;
use crate::model::{EdgePayload, Model};
use crate::nkp::{ends_of, reach};

// ---- configuration ---------------------------------------------------------

/// Incidence analysis tunables (`archi/requirements/scoring/the-matrix-joins-stress-to-structure.md`).
#[derive(Clone, Debug)]
pub struct IncidenceConfig {
    /// Jaccard threshold over two columns' stressor sets for the coupling /
    /// merge findings.
    pub tau_j: f64,
    /// Column density (hits / S) threshold for a stress hotspot.
    pub tau_d: f64,
    /// Matrix density (ones / S×N) threshold for the density alert.
    pub tau_k: f64,
    /// How many declared-edge hops still count as "really connected" when
    /// splitting coupling findings from merge candidates.
    pub depth: usize,
    /// Node budget per connectivity probe; an exhausted budget assumes the
    /// pair connected (suppressing the finding) and warns `PATH_LIMIT_HIT`.
    pub path_limit: usize,
    /// Widen the under-stressed sweep to every zero column. Off, the sweep
    /// names behavior only: terms in the `type_of` closure of `Data` — the
    /// boundary NKP's default slice draws — emit no finding
    /// (`archi/requirements/scoring/findings-read-the-matrix.md`). The matrix and every
    /// other finding always see all columns.
    pub all_terms: bool,
}

impl Default for IncidenceConfig {
    fn default() -> Self {
        IncidenceConfig {
            tau_j: 0.8,
            tau_d: 0.5,
            tau_k: 0.5,
            depth: 2,
            path_limit: 4096,
            all_terms: false,
        }
    }
}

// ---- input -----------------------------------------------------------------

/// A stressor's recorded outcome (`archi/requirements/spec-docs/a-stressor-presses-one-hypothesis.md`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StressOutcome {
    /// The session has not decided yet.
    Pending,
    /// The architecture held.
    Surviving,
    /// The architecture bent; requirements were derived.
    Breaking,
}

impl StressOutcome {
    /// The outcome's lowercase name.
    pub fn describe(self) -> &'static str {
        match self {
            StressOutcome::Pending => "pending",
            StressOutcome::Surviving => "surviving",
            StressOutcome::Breaking => "breaking",
        }
    }
}

/// One matrix row: a stressor whose affects the caller already expanded to
/// term paths against the session's own pinned version.
#[derive(Clone, Debug)]
pub struct IncidenceRow {
    /// The stressor's slug — unique project-wide.
    pub id: String,
    /// Absolute term paths of its pressure surface.
    pub terms: Vec<String>,
    /// Its outcome.
    pub outcome: StressOutcome,
}

/// An invariant of the initial problem statement: the satisfaction claim of
/// an intent-origin requirement (`archi/requirements/spec-docs/satisfaction-is-a-checked-claim.md`).
/// Elements are raw `satisfied-by` paths — terms or types — expanded here
/// against the frame; elements the frame does not know are ignored.
#[derive(Clone, Debug)]
pub struct Invariant {
    /// The requirement's slug.
    pub id: String,
    /// Its `satisfied-by` entries, unexpanded.
    pub elements: Vec<String>,
}

// ---- report ----------------------------------------------------------------

/// Finding severity, ordered: `info` < `warn` < `alert`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Worth a look.
    Info,
    /// Worth an answer.
    Warn,
    /// Worth stopping for.
    Alert,
}

impl Severity {
    /// Parse the lowercase name; `None` for anything else.
    pub fn parse(s: &str) -> Option<Severity> {
        match s {
            "info" => Some(Severity::Info),
            "warn" => Some(Severity::Warn),
            "alert" => Some(Severity::Alert),
            _ => None,
        }
    }

    /// The severity's lowercase name.
    pub fn describe(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warn => "warn",
            Severity::Alert => "alert",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.describe())
    }
}

/// What an incidence finding says. Kinds are append-only; the serialized
/// `kind` tag is the `--kind` filter currency.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IncidenceKind {
    /// Near-identical stress response with no declared path between the
    /// nodes: a hidden dependency.
    HyperliminalCoupling {
        /// One node of the pair (creation order).
        a: String,
        /// The other.
        b: String,
        /// Jaccard similarity of the two stressor sets.
        jaccard: f64,
        /// The shared stressors.
        shared: Vec<String>,
    },
    /// A column pressed by a disproportionate share of the scope.
    StressHotspot {
        /// The node.
        node: String,
        /// How many stressors press it.
        hits: usize,
        /// hits / S.
        density: f64,
        /// The pressing stressors.
        stressors: Vec<String>,
    },
    /// Two surviving stressors, neither covering the invariant alone, whose
    /// union of affected terms does.
    CompoundVulnerability {
        /// The pair, in row order.
        stressors: [String; 2],
        /// The invariant — an intent-origin requirement's slug.
        invariant: String,
        /// The invariant's terms, all affected by the union.
        covered: Vec<String>,
    },
    /// A zero column: no stressor in scope has touched the term.
    UnderStressed {
        /// The node.
        node: String,
    },
    /// Near-identical stress response over a declared path: the pair may
    /// really be one node, or share an extractable concern.
    MergeCandidate {
        /// One node of the pair (creation order).
        a: String,
        /// The other.
        b: String,
        /// Jaccard similarity of the two stressor sets.
        jaccard: f64,
        /// The shared stressors.
        shared: Vec<String>,
    },
    /// The matrix is denser than τ_K: stress is landing everywhere at
    /// once.
    DensityAlert {
        /// Matrix density, ones / (S×N).
        #[serde(rename = "K_hyper")]
        k_hyper: f64,
    },
    /// A row pressing far more of the frame than typical: the stressor
    /// likely crosses a boundary the architecture should make explicit.
    BoundaryCrossingStressor {
        /// The stressor's slug.
        stressor: String,
        /// How many frame terms it presses.
        touches: usize,
        /// The bar it cleared: w̄ + σ over the scope's row weights.
        typical: f64,
        /// The pressed terms.
        terms: Vec<String>,
    },
}

impl IncidenceKind {
    /// The serialized tag, the `--kind` filter currency.
    pub fn name(&self) -> &'static str {
        match self {
            IncidenceKind::HyperliminalCoupling { .. } => "hyperliminal_coupling",
            IncidenceKind::StressHotspot { .. } => "stress_hotspot",
            IncidenceKind::CompoundVulnerability { .. } => "compound_vulnerability",
            IncidenceKind::UnderStressed { .. } => "under_stressed",
            IncidenceKind::MergeCandidate { .. } => "merge_candidate",
            IncidenceKind::DensityAlert { .. } => "density_alert",
            IncidenceKind::BoundaryCrossingStressor { .. } => "boundary_crossing_stressor",
        }
    }
}

/// One typed finding with its severity.
#[derive(Clone, Debug, Serialize)]
pub struct IncidenceFinding {
    /// info / warn / alert.
    pub severity: Severity,
    /// What it says.
    #[serde(flatten)]
    pub kind: IncidenceKind,
}

impl std::fmt::Display for IncidenceFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] ", self.severity)?;
        match &self.kind {
            IncidenceKind::HyperliminalCoupling {
                a,
                b,
                jaccard,
                shared,
            } => write!(
                f,
                "hyperliminal coupling: `{a}` and `{b}` react together across {} stressor(s) (J = {jaccard:.2}) with no declared path between them",
                shared.len()
            ),
            IncidenceKind::StressHotspot {
                node,
                hits,
                density,
                ..
            } => write!(
                f,
                "stress hotspot: `{node}` — pressed by {hits} stressor(s) (density {density:.2})"
            ),
            IncidenceKind::CompoundVulnerability {
                stressors,
                invariant,
                covered,
            } => write!(
                f,
                "compound vulnerability: surviving `{}` + `{}` together cover every element satisfying `{invariant}` ({})",
                stressors[0],
                stressors[1],
                covered.join(", ")
            ),
            IncidenceKind::UnderStressed { node } => {
                write!(f, "under-stressed: `{node}` — no stressor touches it")
            }
            IncidenceKind::MergeCandidate {
                a,
                b,
                jaccard,
                shared,
            } => write!(
                f,
                "merge candidate: `{a}` and `{b}` — near-identical stress response (J = {jaccard:.2}, {} shared stressor(s)) over a declared path",
                shared.len()
            ),
            IncidenceKind::DensityAlert { k_hyper } => write!(
                f,
                "density alert: stress is landing everywhere (K_hyper = {k_hyper:.3})"
            ),
            IncidenceKind::BoundaryCrossingStressor {
                stressor,
                touches,
                typical,
                ..
            } => write!(
                f,
                "boundary-crossing stressor: `{stressor}` — presses {touches} term(s) (w̄+σ = {typical:.2})"
            ),
        }
    }
}

/// The S×N matrix. `stressors` and `outcomes` are parallel; `rows[i][j] = 1`
/// iff stressor `i` presses component `j`.
#[derive(Clone, Debug, Serialize)]
pub struct IncidenceMatrix {
    /// Row order: stressor slugs, caller (session, then file) order.
    pub stressors: Vec<String>,
    /// Each row's outcome, parallel to `stressors`.
    pub outcomes: Vec<StressOutcome>,
    /// Column order: term paths, node-creation order.
    pub components: Vec<String>,
    /// S×N cells.
    pub rows: Vec<Vec<u8>>,
}

/// What the analysis ran over.
#[derive(Clone, Debug, Serialize)]
pub struct IncidenceScope {
    /// S — stressors in scope.
    pub stressor_count: usize,
    /// N — epistatic terms of the frame.
    pub component_count: usize,
    /// Matrix density: ones / (S×N), rounded to three decimals.
    #[serde(rename = "K_hyper")]
    pub k_hyper: f64,
    /// Jaccard threshold used.
    pub tau_j: f64,
    /// Hotspot density threshold used.
    pub tau_d: f64,
    /// Density-alert threshold used.
    pub tau_k: f64,
    /// Connectivity hop bound used.
    pub depth: usize,
    /// Connectivity node budget used.
    pub path_limit: usize,
    /// Affected paths that are not terms of the frame, per stressor —
    /// version drift, kept visible.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub dropped: BTreeMap<String, Vec<String>>,
}

/// A non-fatal analysis warning.
#[derive(Clone, Debug, Serialize)]
pub struct IncidenceWarning {
    /// Stable code: `NO_STRESSORS`, `NO_COMPONENTS`, `DROPPED_AFFECTS`,
    /// `PATH_LIMIT_HIT` or `NO_INVARIANTS`.
    pub code: &'static str,
    /// Human-readable one-liner.
    pub message: String,
}

/// The incidence report — self-contained and renderable independently of
/// the model store.
#[derive(Clone, Debug, Serialize)]
pub struct IncidenceReport {
    /// What the analysis ran over.
    pub scope: IncidenceScope,
    /// The S×N matrix.
    pub matrix: IncidenceMatrix,
    /// Typed findings, severest first.
    pub findings: Vec<IncidenceFinding>,
    /// Analysis warnings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<IncidenceWarning>,
}

// ---- expansion -------------------------------------------------------------

/// The pressure surface of an absolute path: a term is itself; a type
/// expands to the user terms its `type_of` closure classifies. `None` when
/// the path resolves to nothing. Backs [`Model::term_surface`] — the one
/// expansion rule stressor affects and `satisfied-by` entries share
/// (`archi/requirements/spec-docs/`, `archi/requirements/spec-docs/satisfaction-is-a-checked-claim.md`).
pub(crate) fn term_surface(model: &Model, path: &str) -> Option<Vec<String>> {
    let segs: Vec<String> = path.split('.').map(str::to_string).collect();
    let node = model.resolve_in(None, &segs)?;
    let types = ends_of(model, model.type_of, true);
    if !types.contains(&node) {
        return Some(vec![model.node_path(node)]);
    }
    Some(
        reach(model, model.type_of, node, true)
            .into_iter()
            .filter(|n| !types.contains(n) && !model.is_stdlib(n.raw()))
            .map(|n| model.node_path(n))
            .collect(),
    )
}

// ---- analysis --------------------------------------------------------------

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Undirected declared-structure adjacency over the frame's terms: relation
/// edges (a `type_of` edge never qualifies — its source is a type),
/// connection edges between the port-owning nodes, applications between the
/// delegating and realizing nodes, and containment steps. Carriers are
/// metadata, never adjacency.
fn adjacency(model: &Model, columns: &BTreeSet<NodeId>) -> BTreeMap<NodeId, BTreeSet<NodeId>> {
    let mut adj: BTreeMap<NodeId, BTreeSet<NodeId>> =
        columns.iter().map(|&c| (c, BTreeSet::new())).collect();
    let link = |a: NodeId, b: NodeId, adj: &mut BTreeMap<NodeId, BTreeSet<NodeId>>| {
        if a != b && adj.contains_key(&a) && adj.contains_key(&b) {
            adj.get_mut(&a).expect("column exists").insert(b);
            adj.get_mut(&b).expect("column exists").insert(a);
        }
    };
    for e in model.edges.values() {
        match &e.payload {
            EdgePayload::Rel { src, dst, .. } => link(*src, *dst, &mut adj),
            EdgePayload::Conn {
                src_port, dst_port, ..
            } => link(
                model.ports[src_port].node,
                model.ports[dst_port].node,
                &mut adj,
            ),
            EdgePayload::App { outer, inner, .. } => link(
                model.ports[outer].node,
                model.ports[inner].node,
                &mut adj,
            ),
        }
    }
    for (&id, node) in &model.nodes {
        if let Some(p) = node.parent {
            link(id, p, &mut adj);
        }
    }
    adj
}

/// Whether `b` is within `depth` hops of `a`. An exhausted node budget sets
/// `hit` and answers `true` — assuming connection suppresses a finding
/// rather than fabricating one.
fn connected(
    adj: &BTreeMap<NodeId, BTreeSet<NodeId>>,
    a: NodeId,
    b: NodeId,
    depth: usize,
    mut budget: usize,
    hit: &mut bool,
) -> bool {
    let mut seen = BTreeSet::from([a]);
    let mut frontier = vec![a];
    for _ in 0..depth {
        let mut next = Vec::new();
        for x in frontier {
            for &y in &adj[&x] {
                if y == b {
                    return true;
                }
                if seen.insert(y) {
                    if budget == 0 {
                        *hit = true;
                        return true;
                    }
                    budget -= 1;
                    next.push(y);
                }
            }
        }
        if next.is_empty() {
            return false;
        }
        frontier = next;
    }
    false
}

pub(crate) fn analyze(
    model: &Model,
    rows: &[IncidenceRow],
    invariants: &[Invariant],
    config: &IncidenceConfig,
) -> IncidenceReport {
    let mut warnings: Vec<IncidenceWarning> = Vec::new();

    // Columns: the frame's user (non-stdlib) epistatic terms, every scope,
    // in creation order. A type is exactly a `type_of` source.
    let types = ends_of(model, model.type_of, true);
    let columns: Vec<NodeId> = model
        .nodes
        .keys()
        .filter(|n| !model.is_stdlib(n.raw()) && !types.contains(n))
        .copied()
        .collect();
    let paths: Vec<String> = columns.iter().map(|&c| model.node_path(c)).collect();
    let index: BTreeMap<&str, usize> = paths
        .iter()
        .enumerate()
        .map(|(j, p)| (p.as_str(), j))
        .collect();
    let (s, n) = (rows.len(), columns.len());

    // Cells: per-row column sets; paths the frame does not know land in
    // `dropped`.
    let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let cells: Vec<BTreeSet<usize>> = rows
        .iter()
        .map(|row| {
            let mut set = BTreeSet::new();
            let mut lost = BTreeSet::new();
            for t in &row.terms {
                match index.get(t.as_str()) {
                    Some(&j) => {
                        set.insert(j);
                    }
                    None => {
                        lost.insert(t.clone());
                    }
                }
            }
            if !lost.is_empty() {
                dropped.entry(row.id.clone()).or_default().extend(lost);
            }
            set
        })
        .collect();
    let hits: Vec<BTreeSet<usize>> = (0..n)
        .map(|j| (0..s).filter(|&i| cells[i].contains(&j)).collect())
        .collect();
    let ones: usize = cells.iter().map(BTreeSet::len).sum();
    let k_hyper = if s * n == 0 {
        0.0
    } else {
        round3(ones as f64 / (s * n) as f64)
    };

    let mut findings: Vec<IncidenceFinding> = Vec::new();

    // The under-stressed sweep names behavior: zero columns classified
    // under `Data` stay silent unless the config widens the sweep. Only
    // the emission consults this set — a pressed data column counts in the
    // matrix and every other finding regardless. A term named `Data` that
    // classifies nothing is no ontology: nothing is muted.
    let muted: BTreeSet<usize> = if config.all_terms {
        BTreeSet::new()
    } else {
        model
            .resolve_in(None, &["Data".to_string()])
            .filter(|n| types.contains(n))
            .map(|n| {
                reach(model, model.type_of, n, true)
                    .into_iter()
                    .filter_map(|m| index.get(model.node_path(m).as_str()).copied())
                    .collect()
            })
            .unwrap_or_default()
    };

    // Hotspots and zero columns.
    for (j, hit) in hits.iter().enumerate() {
        if s > 0 && hit.is_empty() && !muted.contains(&j) {
            findings.push(IncidenceFinding {
                severity: Severity::Info,
                kind: IncidenceKind::UnderStressed {
                    node: paths[j].clone(),
                },
            });
        }
        if hit.len() >= 2 && hit.len() as f64 / s as f64 >= config.tau_d {
            findings.push(IncidenceFinding {
                severity: Severity::Warn,
                kind: IncidenceKind::StressHotspot {
                    node: paths[j].clone(),
                    hits: hit.len(),
                    density: round3(hit.len() as f64 / s as f64),
                    stressors: hit.iter().map(|&i| rows[i].id.clone()).collect(),
                },
            });
        }
    }

    // Scope-wide pressure, over ≥ 3 rows — three make a distribution. A
    // matrix denser than τ_K alarms: stress is landing everywhere at once.
    // A row pressing far more of the frame than typical — w > w̄ + σ, the
    // NKP hotspot gate turned sideways, alert past 2σ — likely crosses a
    // boundary the architecture should make explicit; the floor mirrors
    // the hotspot's, since a row pressing one term crosses nothing.
    // Weights read the matrix, so dropped affects never widen a row.
    if s >= 3 {
        if k_hyper > config.tau_k {
            findings.push(IncidenceFinding {
                severity: Severity::Alert,
                kind: IncidenceKind::DensityAlert { k_hyper },
            });
        }
        let weights: Vec<usize> = cells.iter().map(BTreeSet::len).collect();
        let mean = weights.iter().sum::<usize>() as f64 / s as f64;
        let sigma = (weights
            .iter()
            .map(|&w| (w as f64 - mean).powi(2))
            .sum::<f64>()
            / s as f64)
            .sqrt();
        for (i, &w) in weights.iter().enumerate() {
            if w >= 2 && w as f64 > mean + sigma {
                let severity = if w as f64 > mean + 2.0 * sigma {
                    Severity::Alert
                } else {
                    Severity::Warn
                };
                findings.push(IncidenceFinding {
                    severity,
                    kind: IncidenceKind::BoundaryCrossingStressor {
                        stressor: rows[i].id.clone(),
                        touches: w,
                        typical: round3(mean + sigma),
                        terms: cells[i].iter().map(|&j| paths[j].clone()).collect(),
                    },
                });
            }
        }
    }

    // Column pairs: near-identical stress response, split by declared-path
    // connectivity into hidden coupling vs merge candidates.
    let column_set: BTreeSet<NodeId> = columns.iter().copied().collect();
    let adj = adjacency(model, &column_set);
    let mut budget_hit = false;
    for a in 0..n {
        for b in a + 1..n {
            let (ha, hb) = (&hits[a], &hits[b]);
            if ha.is_empty() || hb.is_empty() {
                continue;
            }
            let shared: Vec<usize> = ha.intersection(hb).copied().collect();
            if shared.len() < 2 {
                continue;
            }
            let jaccard = shared.len() as f64 / ha.union(hb).count() as f64;
            if jaccard < config.tau_j {
                continue;
            }
            let shared: Vec<String> = shared.iter().map(|&i| rows[i].id.clone()).collect();
            let linked = connected(
                &adj,
                columns[a],
                columns[b],
                config.depth,
                config.path_limit,
                &mut budget_hit,
            );
            let (severity, kind) = if linked {
                (
                    Severity::Info,
                    IncidenceKind::MergeCandidate {
                        a: paths[a].clone(),
                        b: paths[b].clone(),
                        jaccard: round3(jaccard),
                        shared,
                    },
                )
            } else {
                (
                    Severity::Warn,
                    IncidenceKind::HyperliminalCoupling {
                        a: paths[a].clone(),
                        b: paths[b].clone(),
                        jaccard: round3(jaccard),
                        shared,
                    },
                )
            };
            findings.push(IncidenceFinding { severity, kind });
        }
    }

    // Compound vulnerabilities: over surviving pairs only — a breaking
    // stressor already bent the architecture and derived its requirements;
    // a pending one has no verdict to compound.
    let expanded: Vec<(&Invariant, BTreeSet<usize>)> = invariants
        .iter()
        .map(|inv| {
            let set: BTreeSet<usize> = inv
                .elements
                .iter()
                .filter_map(|el| term_surface(model, el))
                .flatten()
                .filter_map(|p| index.get(p.as_str()).copied())
                .collect();
            (inv, set)
        })
        .filter(|(_, set)| !set.is_empty())
        .collect();
    let surviving: Vec<usize> = (0..s)
        .filter(|&i| rows[i].outcome == StressOutcome::Surviving)
        .collect();
    for (x, &i) in surviving.iter().enumerate() {
        for &j in &surviving[x + 1..] {
            for (inv, set) in &expanded {
                let alone = |c: &BTreeSet<usize>| set.iter().all(|t| c.contains(t));
                if !alone(&cells[i])
                    && !alone(&cells[j])
                    && set.iter().all(|t| cells[i].contains(t) || cells[j].contains(t))
                {
                    findings.push(IncidenceFinding {
                        severity: Severity::Alert,
                        kind: IncidenceKind::CompoundVulnerability {
                            stressors: [rows[i].id.clone(), rows[j].id.clone()],
                            invariant: inv.id.clone(),
                            covered: set.iter().map(|&t| paths[t].clone()).collect(),
                        },
                    });
                }
            }
        }
    }

    findings.sort_by(|x, y| {
        (std::cmp::Reverse(x.severity), x.kind.name(), sort_key(&x.kind)).cmp(&(
            std::cmp::Reverse(y.severity),
            y.kind.name(),
            sort_key(&y.kind),
        ))
    });

    // Warnings.
    if s == 0 {
        warnings.push(IncidenceWarning {
            code: "NO_STRESSORS",
            message: "the scope holds no stressors; the matrix is empty".to_string(),
        });
    }
    if n == 0 {
        warnings.push(IncidenceWarning {
            code: "NO_COMPONENTS",
            message: "the frame holds no epistatic terms; the matrix is empty".to_string(),
        });
    }
    if !dropped.is_empty() {
        let count: usize = dropped.values().map(Vec::len).sum();
        warnings.push(IncidenceWarning {
            code: "DROPPED_AFFECTS",
            message: format!(
                "{count} affected path(s) are not terms of the frame; see scope.dropped"
            ),
        });
    }
    if budget_hit {
        warnings.push(IncidenceWarning {
            code: "PATH_LIMIT_HIT",
            message: format!(
                "a connectivity probe exhausted --path-limit {}; such pairs are assumed connected",
                config.path_limit
            ),
        });
    }
    if surviving.len() >= 2 && expanded.is_empty() {
        warnings.push(IncidenceWarning {
            code: "NO_INVARIANTS",
            message:
                "no intent-origin satisfaction claim resolves in the frame; compound vulnerabilities were not tested"
                    .to_string(),
        });
    }

    IncidenceReport {
        scope: IncidenceScope {
            stressor_count: s,
            component_count: n,
            k_hyper,
            tau_j: config.tau_j,
            tau_d: config.tau_d,
            tau_k: config.tau_k,
            depth: config.depth,
            path_limit: config.path_limit,
            dropped,
        },
        matrix: IncidenceMatrix {
            stressors: rows.iter().map(|r| r.id.clone()).collect(),
            outcomes: rows.iter().map(|r| r.outcome).collect(),
            components: paths.clone(),
            rows: cells
                .iter()
                .map(|set| {
                    let mut row = vec![0u8; n];
                    for &j in set {
                        row[j] = 1;
                    }
                    row
                })
                .collect(),
        },
        findings,
        warnings,
    }
}

/// A finding's primary identifiers, for the deterministic sort within one
/// (severity, kind) group.
fn sort_key(kind: &IncidenceKind) -> String {
    match kind {
        IncidenceKind::HyperliminalCoupling { a, b, .. }
        | IncidenceKind::MergeCandidate { a, b, .. } => format!("{a} {b}"),
        IncidenceKind::StressHotspot { node, .. } | IncidenceKind::UnderStressed { node } => {
            node.clone()
        }
        IncidenceKind::CompoundVulnerability {
            stressors,
            invariant,
            ..
        } => format!("{} {} {invariant}", stressors[0], stressors[1]),
        IncidenceKind::DensityAlert { .. } => String::new(),
        IncidenceKind::BoundaryCrossingStressor { stressor, .. } => stressor.clone(),
    }
}
