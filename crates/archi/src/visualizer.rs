//! `archi viz` — an ASCII visualizer for query subgraphs.
//!
//! A subgraph query (`archi query …`) emits a node-link graph as JSON
//! (`Outcome::Graph`); piping it into `archi viz` draws it as an ASCII
//! diagram. The layout engine is the zero-dependency `ascii-dag` crate (a
//! Sugiyama layered layout); this module owns everything around it that keeps
//! the picture *readable*, because a picture that isn't readable is worse than
//! none:
//!
//! - **Collapse deep nesting.** Node ids are absolute dot-paths; a node deeper
//!   than the depth budget folds into its surviving ancestor, which carries a
//!   `+N` badge for the descendants it swallowed.
//! - **Collapse non-mandatory detail.** The diagram shows only the mandatory
//!   structure — node identity and the edges between nodes. Ports, connection
//!   types, carriers, views and prose are withheld unless `--details` asks for
//!   the supplementary listing.
//! - **Break cycles.** `ascii-dag` draws DAGs; architecture graphs cycle
//!   (bidirectional connections, feedback). A back-edge that would cycle is
//!   pulled out of the drawing and reported beneath it, so nothing is lost.
//! - **Refuse the unreadable.** A slice still too large after collapsing is
//!   declined with a summary of where the weight sits and how to narrow it,
//!   rather than drawn as an unreadable tangle.

use std::collections::{BTreeMap, BTreeSet};

use ascii_dag::graph::Graph;
use modeling_lang::EdgeKind;
use serde::Deserialize;
use serde_json::Value;

/// Readability budget. Defaults are tuned so a diagram that renders is always
/// legible; `viz` collapses toward these and refuses past them.
pub struct VizOptions {
    /// Path-segment budget: nodes deeper than this fold into their ancestor.
    pub depth: usize,
    /// The most nodes a diagram may draw before `viz` refuses it.
    pub max_nodes: usize,
    /// The most edges a diagram may draw before `viz` refuses it.
    pub max_edges: usize,
    /// Append the supplementary listing of ports, types, carriers and views.
    pub details: bool,
}

/// The default depth budget: three path segments keeps the widest scope
/// legible while still distinguishing siblings.
pub const DEFAULT_DEPTH: usize = 3;
/// The default node ceiling — past this a layered layout stops being scannable.
pub const DEFAULT_MAX_NODES: usize = 20;
/// The default edge ceiling — past this a layered layout routes into a
/// hairball no matter how few nodes it connects.
pub const DEFAULT_MAX_EDGES: usize = 30;
/// Labels longer than this are truncated to their tail (the most specific,
/// disambiguating segment).
const MAX_LABEL: usize = 28;

impl Default for VizOptions {
    fn default() -> Self {
        VizOptions {
            depth: DEFAULT_DEPTH,
            max_nodes: DEFAULT_MAX_NODES,
            max_edges: DEFAULT_MAX_EDGES,
            details: false,
        }
    }
}

/// One node of the piped graph. Mirrors `modeling_lang::GraphNode`, but owned
/// and `Deserialize` (the library type is serialize-only, and its port `side`
/// is a `&'static str` that cannot be deserialized). Unknown fields are
/// ignored, so a richer producer still parses.
#[derive(Debug, Deserialize)]
pub struct InNode {
    /// Absolute dot-path; the stable id edges reference.
    pub id: String,
    #[serde(default)]
    doc: Option<String>,
    #[serde(default)]
    types: Vec<String>,
}

/// One edge of the piped graph. Mirrors `modeling_lang::GraphEdge`.
#[derive(Debug, Deserialize)]
pub struct InEdge {
    #[serde(default = "default_kind")]
    kind: EdgeKind,
    #[serde(rename = "type", default)]
    type_name: Option<String>,
    #[serde(default)]
    directed: Option<bool>,
    /// Source node id (dot-path).
    pub source: String,
    #[serde(default)]
    source_port: Option<String>,
    /// Target node id (dot-path).
    pub target: String,
    #[serde(default)]
    target_port: Option<String>,
    #[serde(default)]
    carrier: Option<String>,
    #[serde(default)]
    rev_carrier: Option<String>,
    #[serde(default)]
    views: Vec<String>,
}

/// The whole piped graph: the payload of an `Outcome::Graph`.
#[derive(Debug, Deserialize)]
pub struct InGraph {
    #[serde(default)]
    nodes: Vec<InNode>,
    #[serde(default)]
    edges: Vec<InEdge>,
}

/// A bare edge with no `kind` is treated as a relation — the least-committal
/// kind — so a hand-rolled `{nodes, edges}` still renders.
fn default_kind() -> EdgeKind {
    EdgeKind::Relation
}

/// Locate the graph inside whatever was piped in and deserialize it. Accepts
/// the three shapes an `archi` read can produce: the bare `{nodes, edges}` an
/// `Outcome::Graph` serializes to (what `archi query` prints), and the full
/// `{status, results: [...]}` envelope `archi read` prints — the first
/// `graph` result is taken. Anything else is a clear error, not a panic.
pub fn parse_graph(value: &Value) -> Result<InGraph, String> {
    let object = value
        .as_object()
        .ok_or("expected a query graph — a JSON object with `nodes` and `edges`")?;
    let graph = if object.contains_key("nodes") {
        value
    } else if let Some(results) = object.get("results").and_then(Value::as_array) {
        results
            .iter()
            .find(|r| r.get("nodes").is_some())
            .ok_or("the piped response carries no `graph` result to visualize")?
    } else {
        return Err(
            "expected a query graph — pipe `archi query …` (or an `archi read` graph result) \
             into `archi viz`"
                .into(),
        );
    };
    serde_json::from_value(graph.clone()).map_err(|e| format!("the piped graph is malformed: {e}"))
}

/// Render the subgraph as an ASCII diagram, or `Err` with an actionable
/// summary when it is too large to draw readably. An empty slice is not an
/// error — it renders as a one-line note.
pub fn render(graph: &InGraph, opts: &VizOptions) -> Result<String, String> {
    let depth = opts.depth.max(1);

    // Intern collapsed ids in first-appearance order, and count the hidden
    // descendants each surviving ancestor swallowed.
    let mut order: Vec<String> = Vec::new();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    let mut folded: BTreeMap<usize, usize> = BTreeMap::new();
    for node in &graph.nodes {
        let cid = collapse(&node.id, depth);
        let hidden = cid != node.id;
        let i = intern(&mut order, &mut index, cid);
        if hidden {
            *folded.entry(i).or_default() += 1;
        }
    }

    // Collapse edges onto surviving nodes: drop edges that fold into a single
    // node, dedup parallel edges, and remember genuine self-references.
    let mut cedges: Vec<(usize, usize)> = Vec::new();
    let mut seen: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut self_refs: BTreeSet<usize> = BTreeSet::new();
    for edge in &graph.edges {
        let cs = collapse(&edge.source, depth);
        let cd = collapse(&edge.target, depth);
        let i = intern(&mut order, &mut index, cs);
        let j = intern(&mut order, &mut index, cd);
        if i == j {
            if edge.source == edge.target {
                self_refs.insert(i);
            }
            continue;
        }
        if seen.insert((i, j)) {
            cedges.push((i, j));
        }
    }

    if order.is_empty() {
        return Ok("(empty subgraph — the query matched no nodes)\n".into());
    }

    // An unconnected node adds width but no structure — a query's related
    // carrier nodes arrive as isolated boxes. Once there are edges to draw,
    // isolated nodes move to a footnote rather than a sprawling row; with no
    // edges at all, the nodes themselves are the diagram.
    let mut connected: BTreeSet<usize> = BTreeSet::new();
    for &(i, j) in &cedges {
        connected.insert(i);
        connected.insert(j);
    }
    let drawn: Vec<usize> = if cedges.is_empty() {
        (0..order.len()).collect()
    } else {
        connected.iter().copied().collect()
    };
    let isolated: Vec<usize> = if cedges.is_empty() {
        Vec::new()
    } else {
        (0..order.len()).filter(|i| !connected.contains(i)).collect()
    };

    if drawn.len() > opts.max_nodes || cedges.len() > opts.max_edges {
        return Err(too_large(graph, &order, &drawn, cedges.len(), opts));
    }

    // `ascii-dag` draws DAGs; lift back-edges out so the layout is acyclic and
    // report them rather than dropping them silently.
    let (kept, feedback) = break_cycles(order.len(), &cedges);

    // Compact the drawn nodes to a dense 0..k index space for the layout;
    // labels must outlive the borrowing `Graph`, so build the arena first.
    let slot: BTreeMap<usize, usize> =
        drawn.iter().enumerate().map(|(new, &old)| (old, new)).collect();
    let labels: Vec<String> = drawn
        .iter()
        .map(|&old| label_for(&order[old], folded.get(&old).copied().unwrap_or(0)))
        .collect();
    let edges: Vec<(usize, usize)> = kept.iter().map(|&(u, v)| (slot[&u], slot[&v])).collect();
    let diagram = draw(&labels, &edges);

    // Assemble: caption, diagram, notes, then the optional detail listing.
    let hidden: usize = drawn.iter().filter_map(|old| folded.get(old)).sum();
    let mut blocks = vec![
        caption(drawn.len(), cedges.len(), hidden, isolated.len(), depth),
        diagram,
    ];
    if let Some(notes) = notes(&order, &feedback, &self_refs, &isolated) {
        blocks.push(notes);
    }
    if opts.details {
        blocks.push(details(graph));
    }
    Ok(format!("{}\n", blocks.join("\n\n")))
}

/// The first `depth` segments of a dot-path; the whole path when it is already
/// shallow enough.
fn collapse(id: &str, depth: usize) -> String {
    id.split('.').take(depth).collect::<Vec<_>>().join(".")
}

/// Assign `cid` a stable index, reusing an existing one.
fn intern(order: &mut Vec<String>, index: &mut BTreeMap<String, usize>, cid: String) -> usize {
    if let Some(&i) = index.get(&cid) {
        return i;
    }
    let i = order.len();
    index.insert(cid.clone(), i);
    order.push(cid);
    i
}

/// A node's box label: its path (truncated to the tail) plus a `+N` badge for
/// collapsed descendants.
fn label_for(cid: &str, folded: usize) -> String {
    let path = truncate_tail(cid, MAX_LABEL);
    if folded > 0 {
        format!("{path} +{folded}")
    } else {
        path
    }
}

/// Keep the last `max` characters, marking the elision with a leading `…`, so
/// the disambiguating leaf of a long path survives.
fn truncate_tail(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let tail: String = s.chars().skip(count - (max - 1)).collect();
    format!("…{tail}")
}

/// Greedily keep edges in appearance order, dropping any whose target already
/// reaches its source — the minimal spanning DAG plus a list of the back-edges
/// left out. Deterministic: the input order decides the tie.
fn break_cycles(n: usize, edges: &[(usize, usize)]) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut kept = Vec::new();
    let mut feedback = Vec::new();
    for &(u, v) in edges {
        if reaches(&adj, v, u) {
            feedback.push((u, v));
        } else {
            adj[u].push(v);
            kept.push((u, v));
        }
    }
    (kept, feedback)
}

/// Whether `from` reaches `to` over the current kept edges (BFS).
fn reaches(adj: &[Vec<usize>], from: usize, to: usize) -> bool {
    if from == to {
        return true;
    }
    let mut seen = vec![false; adj.len()];
    let mut queue = std::collections::VecDeque::from([from]);
    seen[from] = true;
    while let Some(n) = queue.pop_front() {
        for &m in &adj[n] {
            if m == to {
                return true;
            }
            if !seen[m] {
                seen[m] = true;
                queue.push_back(m);
            }
        }
    }
    false
}

/// Hand the reduced graph to `ascii-dag` and tidy its output (trailing
/// whitespace, blank runs). `Auto` mode draws simple chains inline and falls
/// back to a layered layout for anything branchier.
fn draw(labels: &[String], edges: &[(usize, usize)]) -> String {
    let mut g = Graph::new();
    for (i, label) in labels.iter().enumerate() {
        g.add_node(i, label.as_str());
    }
    for &(u, v) in edges {
        g.add_edge(u, v, None);
    }
    let rendered = g.render();
    let mut out = String::new();
    for line in rendered.lines() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out.trim_matches('\n').to_string()
}

/// The one-line caption above the diagram: what was drawn, and what was set
/// aside to keep it readable.
fn caption(nodes: usize, edges: usize, hidden: usize, unconnected: usize, depth: usize) -> String {
    let mut line = format!("subgraph · {} · {}", count(nodes, "node"), count(edges, "edge"));
    if hidden > 0 {
        line.push_str(&format!(" · {} collapsed at depth {depth}", count(hidden, "node")));
    }
    if unconnected > 0 {
        line.push_str(&format!(" · {unconnected} unconnected"));
    }
    line
}

/// Feedback-edge and self-reference notes, or `None` when the diagram stands
/// on its own.
fn notes(
    order: &[String],
    feedback: &[(usize, usize)],
    self_refs: &BTreeSet<usize>,
    isolated: &[usize],
) -> Option<String> {
    let mut lines = Vec::new();
    if !feedback.is_empty() {
        lines.push("feedback edges (not drawn, would cycle):".to_string());
        for &(u, v) in feedback.iter().take(8) {
            lines.push(format!("  {} → {}", order[u], order[v]));
        }
        if feedback.len() > 8 {
            lines.push(format!("  … and {} more", feedback.len() - 8));
        }
    }
    if !self_refs.is_empty() {
        let names: Vec<&str> = self_refs.iter().map(|&i| order[i].as_str()).collect();
        lines.push(format!("self-references: {}", names.join(", ")));
    }
    if !isolated.is_empty() {
        let names: Vec<&str> = isolated.iter().take(12).map(|&i| order[i].as_str()).collect();
        let mut line = format!("unconnected in this slice: {}", names.join(", "));
        if isolated.len() > 12 {
            line.push_str(&format!(", … and {} more", isolated.len() - 12));
        }
        lines.push(line);
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// The supplementary `--details` listing: the attributes the diagram omits.
fn details(graph: &InGraph) -> String {
    let mut lines = vec!["details".to_string()];
    if !graph.edges.is_empty() {
        lines.push("  edges".to_string());
        for e in &graph.edges {
            lines.push(format!("    {}", edge_detail(e)));
        }
    }
    let described: Vec<&InNode> = graph
        .nodes
        .iter()
        .filter(|n| n.doc.is_some() || !n.types.is_empty())
        .collect();
    if !described.is_empty() {
        lines.push("  nodes".to_string());
        for n in described {
            let mut line = format!("    {}", n.id);
            if let Some(doc) = &n.doc {
                line.push_str(&format!("  «{doc}»"));
            }
            if !n.types.is_empty() {
                line.push_str(&format!("  types: {}", n.types.join(", ")));
            }
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// One edge rendered with everything the diagram collapses: endpoints with
/// ports, kind and type, carried nodes, and views.
fn edge_detail(e: &InEdge) -> String {
    let arrow = if e.directed == Some(false) { "↔" } else { "→" };
    let src = port(&e.source, &e.source_port);
    let dst = port(&e.target, &e.target_port);
    let mut line = format!("{src} {arrow} {dst}   {}", e.kind.name());
    if let Some(t) = &e.type_name {
        line.push_str(&format!(":{t}"));
    }
    match (&e.carrier, &e.rev_carrier) {
        (Some(c), Some(rc)) => line.push_str(&format!("  carries →{c} ←{rc}")),
        (Some(c), None) => line.push_str(&format!("  carries {c}")),
        (None, Some(rc)) => line.push_str(&format!("  carries ←{rc}")),
        (None, None) => {}
    }
    if !e.views.is_empty() {
        line.push_str(&format!("  in [{}]", e.views.join(", ")));
    }
    line
}

/// A node id with an optional `.port` suffix.
fn port(node: &str, port: &Option<String>) -> String {
    match port {
        Some(p) => format!("{node}.{p}"),
        None => node.to_string(),
    }
}

/// The refusal shown when a slice cannot be drawn readably: what tripped the
/// limit, where the weight sits, and the exact commands that narrow it.
fn too_large(
    graph: &InGraph,
    order: &[String],
    drawn: &[usize],
    edge_count: usize,
    opts: &VizOptions,
) -> String {
    let mut out = format!(
        "subgraph too large to visualize readably: {}, {} (after collapsing to depth {}).\n\
         readable limits: ≤{} nodes, ≤{} edges.\n",
        count(drawn.len(), "node"),
        count(edge_count, "edge"),
        opts.depth.max(1),
        opts.max_nodes,
        opts.max_edges,
    );

    // Where the weight sits: node counts per top-level scope, heaviest first.
    let mut scopes: BTreeMap<&str, usize> = BTreeMap::new();
    for &i in drawn {
        let cid = &order[i];
        let top = cid.split('.').next().unwrap_or(cid.as_str());
        *scopes.entry(top).or_default() += 1;
    }
    let mut ranked: Vec<(&str, usize)> = scopes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    if ranked.len() > 1 {
        out.push_str("\ntop-level scopes:\n");
        for (scope, n) in ranked.iter().take(8) {
            out.push_str(&format!("  {scope}   {}\n", count(*n, "node")));
        }
    }

    let (mut rel, mut conn, mut app) = (0, 0, 0);
    for e in &graph.edges {
        match e.kind {
            EdgeKind::Relation => rel += 1,
            EdgeKind::Connection => conn += 1,
            EdgeKind::Application => app += 1,
        }
    }
    let parts: Vec<String> = [(conn, "connection"), (rel, "relation"), (app, "application")]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, k)| count(*n, k))
        .collect();
    if !parts.is_empty() {
        out.push_str(&format!("edge kinds: {}\n", parts.join(", ")));
    }

    out.push_str("\nnarrow the slice and pipe again, e.g.:\n");
    if let Some((scope, _)) = ranked.first() {
        out.push_str(&format!("  archi query --scope {scope} | archi viz\n"));
    }
    out.push_str(
        "  archi query --kind connection | archi viz\n\
         or raise the ceiling: archi viz --max-nodes <n> --depth <n>\n",
    );
    out
}

/// `"1 node"` / `"3 nodes"` — pluralize by appending `s`.
fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(json: Value) -> InGraph {
        parse_graph(&json).expect("valid graph")
    }

    fn viz(json: Value, opts: &VizOptions) -> String {
        render(&graph(json), opts).expect("renders")
    }

    #[test]
    fn draws_a_simple_chain() {
        let out = viz(
            json!({
                "nodes": [{"id": "UI"}, {"id": "AuthService"}, {"id": "TokenStore"}],
                "edges": [
                    {"kind": "connection", "source": "UI", "target": "AuthService"},
                    {"kind": "connection", "source": "AuthService", "target": "TokenStore"}
                ]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("UI"), "{out}");
        assert!(out.contains("AuthService"));
        assert!(out.contains("TokenStore"));
        assert!(out.contains("subgraph · 3 nodes · 2 edges"), "{out}");
    }

    #[test]
    fn parses_the_outcome_graph_envelope() {
        // The tagged `Outcome::Graph` shape that `archi query` prints.
        let g = graph(json!({
            "result": "graph",
            "nodes": [{"id": "A", "name": "A"}],
            "edges": []
        }));
        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn parses_the_read_response_envelope() {
        // The full envelope `archi read` prints; the first graph result is taken.
        let g = graph(json!({
            "status": "ok",
            "results": [{"result": "graph", "nodes": [{"id": "A"}, {"id": "B"}], "edges": []}]
        }));
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn rejects_non_graph_input() {
        let err = parse_graph(&json!({"status": "error"})).unwrap_err();
        assert!(err.contains("archi viz"), "{err}");
    }

    #[test]
    fn collapses_nesting_past_the_depth_budget() {
        let out = viz(
            json!({
                "nodes": [
                    {"id": "Orders"},
                    {"id": "Orders.Handler"},
                    {"id": "Orders.Handler.Inner"},
                    {"id": "Orders.Handler.Inner.Leaf"}
                ],
                "edges": []
            }),
            &VizOptions { depth: 2, ..Default::default() },
        );
        // Orders.Handler.Inner and .Leaf fold into Orders.Handler (+2).
        assert!(out.contains("Orders.Handler +2"), "{out}");
        assert!(!out.contains("Inner"), "{out}");
        assert!(out.contains("collapsed at depth 2"), "{out}");
    }

    #[test]
    fn breaks_cycles_and_reports_feedback_edges() {
        let out = viz(
            json!({
                "nodes": [{"id": "A"}, {"id": "B"}, {"id": "C"}],
                "edges": [
                    {"kind": "relation", "source": "A", "target": "B"},
                    {"kind": "relation", "source": "B", "target": "C"},
                    {"kind": "relation", "source": "C", "target": "A"}
                ]
            }),
            &VizOptions::default(),
        );
        // A real diagram, never ascii-dag's "CYCLE DETECTED" bail-out.
        assert!(!out.contains("CYCLE DETECTED"), "{out}");
        assert!(out.contains("feedback edges"), "{out}");
        assert!(out.contains("C → A"), "{out}");
    }

    #[test]
    fn drops_and_reports_self_references() {
        let out = viz(
            json!({
                "nodes": [{"id": "Cache"}, {"id": "DB"}],
                "edges": [
                    {"kind": "relation", "source": "Cache", "target": "Cache"},
                    {"kind": "relation", "source": "Cache", "target": "DB"}
                ]
            }),
            &VizOptions::default(),
        );
        assert!(!out.contains("CYCLE DETECTED"), "{out}");
        assert!(out.contains("self-references: Cache"), "{out}");
    }

    #[test]
    fn dedupes_parallel_edges() {
        // Two edges between the same pair count once in the diagram.
        let out = viz(
            json!({
                "nodes": [{"id": "A"}, {"id": "B"}],
                "edges": [
                    {"kind": "connection", "source": "A", "target": "B"},
                    {"kind": "relation", "source": "A", "target": "B"}
                ]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("subgraph · 2 nodes · 1 edge"), "{out}");
    }

    #[test]
    fn refuses_when_too_large() {
        let nodes: Vec<Value> = (0..30).map(|i| json!({"id": format!("N{i}")})).collect();
        let err = render(
            &graph(json!({"nodes": nodes, "edges": []})),
            &VizOptions::default(),
        )
        .unwrap_err();
        assert!(err.contains("too large"), "{err}");
        assert!(err.contains("archi query"), "{err}");
        assert!(err.contains("30 nodes"), "{err}");
    }

    #[test]
    fn empty_slice_is_not_an_error() {
        let out = viz(json!({"nodes": [], "edges": []}), &VizOptions::default());
        assert!(out.contains("empty subgraph"), "{out}");
    }

    #[test]
    fn details_lists_ports_types_and_carriers() {
        let out = viz(
            json!({
                "nodes": [
                    {"id": "Orders", "doc": "the order boundary", "types": ["Service"]}
                ],
                "edges": [{
                    "kind": "connection", "type": "send", "directed": true,
                    "source": "Shipping", "source_port": "out",
                    "target": "Orders", "target_port": "events",
                    "carrier": "OrderCreated", "views": ["flow"]
                }]
            }),
            &VizOptions { details: true, ..Default::default() },
        );
        assert!(out.contains("details"), "{out}");
        assert!(out.contains("Shipping.out → Orders.events"), "{out}");
        assert!(out.contains("connection:send"), "{out}");
        assert!(out.contains("carries OrderCreated"), "{out}");
        assert!(out.contains("in [flow]"), "{out}");
        assert!(out.contains("«the order boundary»"), "{out}");
        assert!(out.contains("types: Service"), "{out}");
    }

    #[test]
    fn details_are_withheld_by_default() {
        let out = viz(
            json!({
                "nodes": [{"id": "A"}, {"id": "B"}],
                "edges": [{"kind": "connection", "type": "send", "source": "A", "target": "B"}]
            }),
            &VizOptions::default(),
        );
        assert!(!out.contains("details"), "{out}");
        assert!(!out.contains("send"), "{out}");
    }

    use serde_json::json;
}
