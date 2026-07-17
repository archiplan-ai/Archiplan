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
//! - **Collapse non-mandatory detail.** The diagram shows the mandatory
//!   structure — node identity, the edges between nodes, and each edge's
//!   rel/conn type as a `‹tag›` on its path, so no arrow is anonymous. Ports,
//!   views and prose are withheld unless `--details` asks for the
//!   supplementary listing; so is a carrier whose node the slice leaves out.
//! - **Draw data into the flow.** A connection's payload rides its lanes as
//!   `carrier` (forward) and `rev_carrier` (reverse). When the carried node is
//!   itself in the slice, the edge is drawn *through* it — source → data →
//!   target — so the flow of data is visible and a shared payload becomes the
//!   junction its producers and consumers meet at, not an unconnected box in
//!   a footnote. Data boxes are rounded — `(Data)` against a component's
//!   `[Component]` — and a routed edge carries no type tag: its payload names
//!   the interaction.
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
    //
    // Data flows *via* edges: a payload rides a connection's lanes as
    // `carrier` (forward) and `rev_carrier` (reverse). When the carried node
    // is itself in the slice, the edge is drawn *through* it — source → data →
    // target, and target → data → source for the reverse lane — so a shared
    // payload becomes the junction its producers and consumers meet at. A
    // carrier the slice leaves out stays a `--details` annotation.
    //
    // An edge drawn direct keeps its rel/conn type name as a tag on the path
    // (`A → ‹wire› → B`); a routed edge needs none — its payload names the
    // interaction. Parallel edges of *different* types stay distinct paths.
    let declared: BTreeSet<String> = graph.nodes.iter().map(|n| collapse(&n.id, depth)).collect();
    let mut cedges: Vec<(usize, usize, Option<String>)> = Vec::new();
    let mut seen: BTreeSet<(usize, usize, Option<String>)> = BTreeSet::new();
    let mut self_refs: BTreeSet<usize> = BTreeSet::new();
    let mut data_nodes: BTreeSet<usize> = BTreeSet::new();
    for edge in &graph.edges {
        let fwd = carried(&edge.carrier, &declared, depth);
        let rev = carried(&edge.rev_carrier, &declared, depth);
        let tag = edge.type_name.as_deref().filter(|t| !t.is_empty());
        let mut hops: Vec<(&str, &str, Option<&str>)> = Vec::new();
        match fwd {
            Some(c) => {
                hops.push((&edge.source, c, None));
                hops.push((c, &edge.target, None));
            }
            None => hops.push((&edge.source, &edge.target, tag)),
        }
        if let Some(c) = rev {
            hops.push((&edge.target, c, None));
            hops.push((c, &edge.source, None));
        }
        for (s, t, tag) in hops {
            let i = intern(&mut order, &mut index, collapse(s, depth));
            let j = intern(&mut order, &mut index, collapse(t, depth));
            if i == j {
                if s == t {
                    self_refs.insert(i);
                }
                continue;
            }
            if fwd == Some(s) || rev == Some(s) {
                data_nodes.insert(i);
            }
            if fwd == Some(t) || rev == Some(t) {
                data_nodes.insert(j);
            }
            if seen.insert((i, j, tag.map(String::from))) {
                cedges.push((i, j, tag.map(String::from)));
            }
        }
    }

    if order.is_empty() {
        return Ok("(empty subgraph — the query matched no nodes)\n".into());
    }

    // An unconnected node adds width but no structure. Once there are edges
    // to draw, isolated nodes move to a footnote rather than a sprawling row;
    // with no edges at all, the nodes themselves are the diagram.
    let mut connected: BTreeSet<usize> = BTreeSet::new();
    for (i, j, _) in &cedges {
        connected.insert(*i);
        connected.insert(*j);
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
    // A typed edge routes through a tag pseudo-node (`A → ‹wire› → B`): a tag
    // is a layout node, so its placement is collision-proof — unlike the
    // layout engine's inline edge labels, which drop and mislead on merged
    // lanes. The box is stripped after rendering.
    let slot: BTreeMap<usize, usize> =
        drawn.iter().enumerate().map(|(new, &old)| (old, new)).collect();
    let mut labels: Vec<String> = drawn
        .iter()
        .map(|&old| label_for(&order[old], folded.get(&old).copied().unwrap_or(0)))
        .collect();
    let tags = labels.len();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for (u, v, tag) in &kept {
        match tag {
            Some(t) => {
                let k = labels.len();
                labels.push(format!("‹{}›", truncate_tail(t, MAX_LABEL)));
                edges.push((slot[u], k));
                edges.push((k, slot[v]));
            }
            None => edges.push((slot[u], slot[v])),
        }
    }
    let mut diagram = draw(&labels, &edges);

    // Restyle after layout — `(`, `[` and space are all one column, so
    // alignment is untouched:
    // - a payload junction is rounded, (Data) against a component's
    //   [Component], so the two read apart at a glance;
    // - an edge tag sheds its box, ‹wire› on the path rather than a node.
    for (k, &old) in drawn.iter().enumerate() {
        if data_nodes.contains(&old) {
            diagram = diagram.replace(&format!("[{}]", labels[k]), &format!("({})", labels[k]));
        }
    }
    for tag in &labels[tags..] {
        diagram = diagram.replace(&format!("[{tag}]"), &format!(" {tag} "));
    }
    let diagram = diagram.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");

    // Assemble: caption, diagram, notes, then the optional detail listing.
    let hidden: usize = drawn.iter().filter_map(|old| folded.get(old)).sum();
    let mut blocks = vec![
        caption(drawn.len(), cedges.len(), hidden, isolated.len(), depth),
        diagram,
    ];
    if let Some(notes) = notes(&order, &feedback, &self_refs, &data_nodes, &isolated) {
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

/// The lane's payload, when the carried node is itself in the slice — the
/// condition for drawing the edge through it rather than direct.
fn carried<'a>(
    lane: &'a Option<String>,
    declared: &BTreeSet<String>,
    depth: usize,
) -> Option<&'a str> {
    lane.as_deref().filter(|c| declared.contains(&collapse(c, depth)))
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
#[allow(clippy::type_complexity)]
fn break_cycles(
    n: usize,
    edges: &[(usize, usize, Option<String>)],
) -> (Vec<(usize, usize, Option<String>)>, Vec<(usize, usize, Option<String>)>) {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut kept = Vec::new();
    let mut feedback = Vec::new();
    for (u, v, tag) in edges {
        if reaches(&adj, *v, *u) {
            feedback.push((*u, *v, tag.clone()));
        } else {
            adj[*u].push(*v);
            kept.push((*u, *v, tag.clone()));
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

/// Feedback-edge, self-reference and data-node notes, or `None` when the
/// diagram stands on its own.
fn notes(
    order: &[String],
    feedback: &[(usize, usize, Option<String>)],
    self_refs: &BTreeSet<usize>,
    data_nodes: &BTreeSet<usize>,
    isolated: &[usize],
) -> Option<String> {
    let mut lines = Vec::new();
    if !feedback.is_empty() {
        lines.push("feedback edges (not drawn, would cycle):".to_string());
        for (u, v, tag) in feedback.iter().take(8) {
            let name = tag.as_ref().map(|t| format!("‹{t}› → ")).unwrap_or_default();
            lines.push(format!("  {} → {name}{}", order[*u], order[*v]));
        }
        if feedback.len() > 8 {
            lines.push(format!("  … and {} more", feedback.len() - 8));
        }
    }
    if !self_refs.is_empty() {
        let names: Vec<&str> = self_refs.iter().map(|&i| order[i].as_str()).collect();
        lines.push(format!("self-references: {}", names.join(", ")));
    }
    if !data_nodes.is_empty() {
        let names: Vec<&str> = data_nodes.iter().take(12).map(|&i| order[i].as_str()).collect();
        let mut line = format!("data carried on edges: {}", names.join(", "));
        if data_nodes.len() > 12 {
            line.push_str(&format!(", … and {} more", data_nodes.len() - 12));
        }
        lines.push(line);
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
    fn routes_edges_through_carried_data_nodes() {
        // The payload is in the slice, so the edge is drawn through it:
        // UI → LoginForm → AuthService, not UI → AuthService plus a footnote.
        let out = viz(
            json!({
                "nodes": [{"id": "UI"}, {"id": "AuthService"}, {"id": "LoginForm"}],
                "edges": [{
                    "kind": "connection", "source": "UI", "target": "AuthService",
                    "carrier": "LoginForm"
                }]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("subgraph · 3 nodes · 2 edges"), "{out}");
        // The payload's box is rounded, a component's square.
        assert!(out.contains("(LoginForm)"), "{out}");
        assert!(out.contains("[UI]"), "{out}");
        assert!(out.contains("[AuthService]"), "{out}");
        assert!(out.contains("data carried on edges: LoginForm"), "{out}");
        assert!(!out.contains("unconnected"), "{out}");
    }

    #[test]
    fn reverse_carriers_route_back_through_their_data_node() {
        // A pull: the reverse lane carries Token back. The return hop closes a
        // cycle, so one leg is lifted to the feedback note — never dropped.
        let out = viz(
            json!({
                "nodes": [{"id": "UI"}, {"id": "AuthService"}, {"id": "Token"}],
                "edges": [{
                    "kind": "connection", "source": "UI", "target": "AuthService",
                    "rev_carrier": "Token"
                }]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("subgraph · 3 nodes · 3 edges"), "{out}");
        assert!(out.contains("(Token)"), "{out}");
        assert!(out.contains("data carried on edges: Token"), "{out}");
        assert!(out.contains("feedback edges"), "{out}");
        assert!(out.contains("Token → UI"), "{out}");
        assert!(!out.contains("unconnected"), "{out}");
    }

    #[test]
    fn shared_carriers_become_junctions() {
        // Two producers of the same payload meet at its node; the parallel
        // hops into the shared consumer dedup.
        let out = viz(
            json!({
                "nodes": [
                    {"id": "Rpc"}, {"id": "Stream"}, {"id": "Meter"}, {"id": "UsageRecord"}
                ],
                "edges": [
                    {"kind": "connection", "source": "Rpc", "target": "Meter",
                     "carrier": "UsageRecord"},
                    {"kind": "connection", "source": "Stream", "target": "Meter",
                     "carrier": "UsageRecord"}
                ]
            }),
            &VizOptions::default(),
        );
        // Rpc → UsageRecord, Stream → UsageRecord, UsageRecord → Meter.
        assert!(out.contains("subgraph · 4 nodes · 3 edges"), "{out}");
        assert!(out.contains("(UsageRecord)"), "{out}");
        assert!(out.contains("data carried on edges: UsageRecord"), "{out}");
        assert!(!out.contains("unconnected"), "{out}");
    }

    #[test]
    fn carriers_absent_from_the_slice_stay_out_of_the_diagram() {
        // The payload node was filtered out of the slice: the edge is drawn
        // direct, and the carrier remains a `--details` annotation.
        let out = viz(
            json!({
                "nodes": [{"id": "A"}, {"id": "B"}],
                "edges": [{
                    "kind": "connection", "source": "A", "target": "B", "carrier": "X"
                }]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("subgraph · 2 nodes · 1 edge"), "{out}");
        assert!(!out.contains("X"), "{out}");
        assert!(!out.contains("data carried"), "{out}");
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
                "edges": [{
                    "kind": "connection", "type": "send", "source": "A",
                    "source_port": "out", "target": "B", "views": ["flow"]
                }]
            }),
            &VizOptions::default(),
        );
        // The type is structure and stays — as the edge's tag — but ports and
        // views wait for --details.
        assert!(!out.contains("details"), "{out}");
        assert!(out.contains("‹send›"), "{out}");
        assert!(!out.contains("A.out"), "{out}");
        assert!(!out.contains("flow"), "{out}");
    }

    #[test]
    fn direct_edges_are_tagged_with_their_type() {
        let out = viz(
            json!({
                "nodes": [{"id": "Orders"}, {"id": "Billing"}],
                "edges": [
                    {"kind": "connection", "type": "wire", "source": "Orders", "target": "Billing"}
                ]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("subgraph · 2 nodes · 1 edge"), "{out}");
        assert!(out.contains("‹wire›"), "{out}");
        // The tag rides the path unboxed — never as a node of its own.
        assert!(!out.contains("[‹wire›]"), "{out}");
    }

    #[test]
    fn parallel_edges_of_different_types_stay_distinct() {
        let out = viz(
            json!({
                "nodes": [{"id": "A"}, {"id": "B"}],
                "edges": [
                    {"kind": "connection", "type": "wire", "source": "A", "target": "B"},
                    {"kind": "relation", "type": "audits", "source": "A", "target": "B"}
                ]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("subgraph · 2 nodes · 2 edges"), "{out}");
        assert!(out.contains("‹wire›"), "{out}");
        assert!(out.contains("‹audits›"), "{out}");
    }

    #[test]
    fn routed_edges_carry_no_type_tag() {
        // The payload junction names the interaction; a tag would say it twice.
        let out = viz(
            json!({
                "nodes": [{"id": "UI"}, {"id": "AuthService"}, {"id": "LoginForm"}],
                "edges": [{
                    "kind": "connection", "type": "login", "source": "UI",
                    "target": "AuthService", "carrier": "LoginForm"
                }]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("(LoginForm)"), "{out}");
        assert!(!out.contains("‹login›"), "{out}");
    }

    #[test]
    fn feedback_notes_name_the_edge_type() {
        let out = viz(
            json!({
                "nodes": [{"id": "A"}, {"id": "B"}],
                "edges": [
                    {"kind": "connection", "type": "req", "source": "A", "target": "B"},
                    {"kind": "connection", "type": "ack", "source": "B", "target": "A"}
                ]
            }),
            &VizOptions::default(),
        );
        assert!(out.contains("feedback edges"), "{out}");
        assert!(out.contains("B → ‹ack› → A"), "{out}");
    }

    use serde_json::json;
}
