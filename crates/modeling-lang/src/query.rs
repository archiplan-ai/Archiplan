//! Read statements: the subgraph `query` and `check`.
//!
//! A query slices the model with composed filters — types, kinds, views,
//! scopes — and returns the slice as plain nodes and edges
//! (`requirements/modeling-lang/queries.md`). Views restrict to the edges of
//! the named views; applications are untagged plumbing and belong to the
//! views of the connection edges they route.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ids::{EdgeId, NodeId, PortId, ViewId};
use crate::model::{Edge, EdgePayload, Model, Side};
use crate::result::{Finding, GraphEdge, GraphNode, GraphPort};
use crate::statement::{EdgeKind, Statement};

/// Where a connection edge attached to a delegated port ends up.
pub(crate) enum Route {
    /// No delegation exists on the port; the node is opaque here.
    NoDelegations,
    /// The edge is routed by these applications.
    Routed(Vec<EdgeId>),
    /// Delegations exist but none matches this edge.
    Unrouted,
}

/// Resolve routing of one connection edge across one of its ports: an edge is
/// routed by the qualified delegation whose pattern matches its carried node;
/// edges matched by no qualifier fall back to the unqualified delegation.
pub(crate) fn route(model: &Model, edge: &Edge, port: PortId) -> Route {
    let carrier = match &edge.payload {
        EdgePayload::Conn { carrier, .. } => *carrier,
        _ => return Route::NoDelegations,
    };
    let mut qualified = Vec::new();
    let mut unqualified = Vec::new();
    for app in model.apps_on_outer_port(port) {
        match &app.payload {
            EdgePayload::App {
                qualifier: Some(q), ..
            } => {
                if carrier.is_some_and(|c| model.matches(q, c)) {
                    qualified.push(app.id);
                }
            }
            EdgePayload::App {
                qualifier: None, ..
            } => unqualified.push(app.id),
            _ => {}
        }
    }
    if !qualified.is_empty() {
        return Route::Routed(qualified);
    }
    if !unqualified.is_empty() {
        return Route::Routed(unqualified);
    }
    if model.apps_on_outer_port(port).next().is_some() {
        Route::Unrouted
    } else {
        Route::NoDelegations
    }
}

/// The views an application belongs to: the union of the views of the
/// connection edges it routes.
pub(crate) fn app_views(model: &Model, app: &Edge) -> BTreeSet<ViewId> {
    let outer = match &app.payload {
        EdgePayload::App { outer, .. } => *outer,
        _ => return BTreeSet::new(),
    };
    let mut views = BTreeSet::new();
    for e in model.conn_edges_on_port(outer) {
        if let Route::Routed(apps) = route(model, e, outer)
            && apps.contains(&app.id)
        {
            views.extend(e.views.iter().copied());
        }
    }
    views
}

fn edge_in_filter(model: &Model, e: &Edge, filter: Option<&BTreeSet<ViewId>>) -> bool {
    let Some(f) = filter else { return true };
    match &e.payload {
        EdgePayload::App { .. } => !app_views(model, e).is_disjoint(f),
        _ => !e.views.is_disjoint(f),
    }
}

/// Resolved `query` filters. `None` never restricts; an empty list is the
/// most restrictive filter of its category.
pub(crate) struct SubgraphFilter {
    pub types: Option<Vec<NodeId>>,
    pub kinds: Option<BTreeSet<EdgeKind>>,
    pub views: Option<BTreeSet<ViewId>>,
    pub scopes: Option<Vec<NodeId>>,
}

fn kind_of(e: &Edge) -> EdgeKind {
    match &e.payload {
        EdgePayload::Rel { .. } => EdgeKind::Relation,
        EdgePayload::Conn { .. } => EdgeKind::Connection,
        EdgePayload::App { .. } => EdgeKind::Application,
    }
}

/// The nodes an edge attaches to. The carrier is metadata, not an attachment:
/// it does not decide whether the edge is part of a slice.
fn attachments(model: &Model, e: &Edge) -> [NodeId; 2] {
    match &e.payload {
        EdgePayload::Rel { src, dst, .. } => [*src, *dst],
        EdgePayload::Conn {
            src_port, dst_port, ..
        } => [model.ports[src_port].node, model.ports[dst_port].node],
        EdgePayload::App { outer, inner, .. } => [model.ports[outer].node, model.ports[inner].node],
    }
}

/// The scopes a `scopes` filter opens: for each named node, the chain from
/// the root down to it plus its whole subtree. The top level is always open.
fn opened_scopes(model: &Model, roots: &[NodeId]) -> BTreeSet<NodeId> {
    let mut opened = BTreeSet::new();
    for &root in roots {
        opened.extend(model.scope_chain(root));
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            for &child in model.nodes[&n].children.values() {
                if opened.insert(child) {
                    stack.push(child);
                }
            }
        }
    }
    opened
}

/// Every node classifying `node` via `type_of`, following the transitive
/// closure over declared edges.
fn classifiers(model: &Model, node: NodeId) -> BTreeSet<NodeId> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([node]);
    while let Some(n) = queue.pop_front() {
        for e in model.edges.values() {
            if let EdgePayload::Rel { rel, src, dst } = &e.payload
                && *rel == model.type_of
                && *dst == n
                && seen.insert(*src)
            {
                queue.push_back(*src);
            }
        }
    }
    seen
}

fn view_names(model: &Model, views: impl IntoIterator<Item = ViewId>) -> Vec<String> {
    views
        .into_iter()
        .map(|v| model.views[&v].name.clone())
        .collect()
}

fn graph_edge(model: &Model, e: &Edge) -> GraphEdge {
    let [src, dst] = attachments(model, e);
    let mut out = GraphEdge {
        kind: kind_of(e),
        type_name: None,
        directed: None,
        source: model.node_path(src),
        source_port: None,
        target: model.node_path(dst),
        target_port: None,
        carrier: None,
        route: None,
        views: view_names(model, e.views.iter().copied()),
    };
    match &e.payload {
        EdgePayload::Rel { rel, .. } => {
            let rt = &model.rels[rel];
            out.type_name = Some(rt.name.clone());
            out.directed = Some(rt.directed);
        }
        EdgePayload::Conn {
            conn,
            src_port,
            carrier,
            dst_port,
        } => {
            let ct = &model.conns[conn];
            out.type_name = Some(ct.name.clone());
            out.directed = Some(ct.directed);
            out.source_port = Some(model.ports[src_port].name.clone());
            out.target_port = Some(model.ports[dst_port].name.clone());
            out.carrier = carrier.map(|c| model.node_path(c));
        }
        EdgePayload::App {
            outer,
            qualifier,
            inner,
        } => {
            out.source_port = Some(model.ports[outer].name.clone());
            out.target_port = Some(model.ports[inner].name.clone());
            out.route = qualifier.as_ref().map(|q| model.pattern_expr(q));
            out.views = view_names(model, app_views(model, e));
        }
    }
    out
}

/// Slice the model with composed filters into plain nodes and edges, both in
/// creation order.
///
/// - `types` keeps nodes classified by any listed type (transitively);
/// - `kinds` keeps edges of the listed kinds;
/// - `views` keeps edges of the listed views, and only nodes related to them
///   (their attachments and carriers);
/// - `scopes` keeps the top level plus the named scopes' chains and subtrees;
/// - an edge needs all its attachments in the slice to survive.
pub(crate) fn subgraph(model: &Model, filter: &SubgraphFilter) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let opened = filter
        .scopes
        .as_deref()
        .map(|roots| opened_scopes(model, roots));
    let edge_pass = |e: &Edge| {
        filter
            .kinds
            .as_ref()
            .is_none_or(|ks| ks.contains(&kind_of(e)))
            && edge_in_filter(model, e, filter.views.as_ref())
    };
    // A views filter admits nodes through its edges: attachments and carriers.
    let related: Option<BTreeSet<NodeId>> = filter.views.as_ref().map(|_| {
        model
            .edges
            .values()
            .filter(|e| edge_pass(e))
            .flat_map(|e| {
                let mut nodes = attachments(model, e).to_vec();
                if let EdgePayload::Conn {
                    carrier: Some(c), ..
                } = &e.payload
                {
                    nodes.push(*c);
                }
                nodes
            })
            .collect()
    });
    let node_pass = |n: NodeId| {
        filter
            .types
            .as_ref()
            .is_none_or(|ts| ts.iter().any(|t| model.rel_holds(model.type_of, *t, n)))
            && opened.as_ref().is_none_or(|o| {
                let chain = model.scope_chain(n);
                chain[..chain.len() - 1].iter().all(|a| o.contains(a))
            })
            && related.as_ref().is_none_or(|r| r.contains(&n))
    };
    let included: BTreeSet<NodeId> = model
        .nodes
        .keys()
        .copied()
        .filter(|&n| node_pass(n))
        .collect();

    let mut edges = Vec::new();
    let mut port_refs: BTreeMap<NodeId, BTreeSet<PortId>> = BTreeMap::new();
    for e in model.edges.values() {
        if !edge_pass(e) || !attachments(model, e).iter().all(|n| included.contains(n)) {
            continue;
        }
        let ports = match &e.payload {
            EdgePayload::Conn {
                src_port, dst_port, ..
            } => Some([*src_port, *dst_port]),
            EdgePayload::App { outer, inner, .. } => Some([*outer, *inner]),
            EdgePayload::Rel { .. } => None,
        };
        for p in ports.into_iter().flatten() {
            port_refs.entry(model.ports[&p].node).or_default().insert(p);
        }
        edges.push(graph_edge(model, e));
    }

    let nodes = included
        .iter()
        .map(|&n| GraphNode {
            id: model.node_path(n),
            name: model.nodes[&n].name.clone(),
            types: classifiers(model, n)
                .iter()
                .map(|&t| model.node_path(t))
                .collect(),
            ports: port_refs
                .get(&n)
                .into_iter()
                .flatten()
                .map(|p| {
                    let port = &model.ports[p];
                    GraphPort {
                        name: port.name.clone(),
                        conn: model.conns[&port.conn].name.clone(),
                        side: port.side.map(Side::describe),
                    }
                })
                .collect(),
        })
        .collect();
    (nodes, edges)
}

/// The model rendered as replayable statements, in creation order. Stdlib
/// (preset) elements are omitted: a restore loads the same preset first.
pub(crate) fn dump(model: &Model) -> Vec<Statement> {
    let mut items: Vec<(u64, Statement)> = Vec::new();
    for v in model.views.values() {
        items.push((v.id.raw(), model.view_statement(v)));
    }
    for r in model.rels.values() {
        items.push((r.id.raw(), model.rel_statement(r)));
    }
    for c in model.conns.values() {
        items.push((c.id.raw(), model.conn_statement(c)));
    }
    for n in model.nodes.values() {
        items.push((n.id.raw(), model.node_statement(n.id)));
    }
    for e in model.edges.values() {
        items.push((e.id.raw(), model.edge_statement(e)));
    }
    items.retain(|(id, _)| !model.is_stdlib(*id));
    items.sort_by_key(|(id, _)| *id);
    items.into_iter().map(|(_, s)| s).collect()
}

/// Model-completeness findings. With a view filter, edge-scoped findings are
/// restricted to edges of those views and emptiness is reported only for the
/// named views; type findings are unaffected.
pub(crate) fn check(model: &Model, filter: Option<&BTreeSet<ViewId>>) -> Vec<Finding> {
    let mut out = Vec::new();

    // Shape conformance drift: re-check every edge against its type's patterns.
    for e in model.edges.values() {
        if !edge_in_filter(model, e, filter) {
            continue;
        }
        match &e.payload {
            EdgePayload::Rel { rel, src, dst } => {
                let rt = &model.rels[rel];
                let fits =
                    |a: NodeId, b: NodeId| model.matches(&rt.src, a) && model.matches(&rt.dst, b);
                if !(fits(*src, *dst) || (!rt.directed && fits(*dst, *src))) {
                    let (slot, pat, node) = if !model.matches(&rt.src, *src) {
                        ("source", &rt.src, *src)
                    } else {
                        ("target", &rt.dst, *dst)
                    };
                    out.push(Finding::ShapeDrift {
                        statement: model.edge_statement(e),
                        slot: slot.to_string(),
                        expected: model.pattern_expr(pat),
                        actual: model.node_path(node),
                    });
                }
            }
            EdgePayload::Conn {
                conn,
                src_port,
                carrier,
                dst_port,
            } => {
                let ct = &model.conns[conn];
                let a = model.ports[src_port].node;
                let b = model.ports[dst_port].node;
                let fits =
                    |x: NodeId, y: NodeId| model.matches(&ct.src, x) && model.matches(&ct.dst, y);
                if !(fits(a, b) || (!ct.directed && fits(b, a))) {
                    let (slot, pat, node) = if !model.matches(&ct.src, a) {
                        ("source", &ct.src, a)
                    } else {
                        ("target", &ct.dst, b)
                    };
                    out.push(Finding::ShapeDrift {
                        statement: model.edge_statement(e),
                        slot: slot.to_string(),
                        expected: model.pattern_expr(pat),
                        actual: model.node_path(node),
                    });
                }
                match (&ct.carrier, carrier) {
                    (Some(cp), Some(c)) => {
                        if !model.matches(cp, *c) {
                            out.push(Finding::ShapeDrift {
                                statement: model.edge_statement(e),
                                slot: "carrier".to_string(),
                                expected: model.pattern_expr(cp),
                                actual: model.node_path(*c),
                            });
                        }
                    }
                    // Arity drift after a type redefine: report against the
                    // slot that no longer matches the edge's structure.
                    (Some(cp), None) => {
                        out.push(Finding::ShapeDrift {
                            statement: model.edge_statement(e),
                            slot: "carrier".to_string(),
                            expected: model.pattern_expr(cp),
                            actual: "(none)".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            EdgePayload::App { .. } => {}
        }
    }

    // Carried traffic that matches no delegation.
    for e in model.edges.values() {
        if !edge_in_filter(model, e, filter) {
            continue;
        }
        if let EdgePayload::Conn {
            src_port, dst_port, ..
        } = &e.payload
        {
            for port in [*src_port, *dst_port] {
                if matches!(route(model, e, port), Route::Unrouted) {
                    out.push(Finding::UnroutedTraffic {
                        statement: model.edge_statement(e),
                        port: model.port_path(port),
                    });
                }
            }
        }
    }

    // Delegated ports with no attached connections.
    for p in model.ports.values() {
        if model.apps_on_outer_port(p.id).next().is_some()
            && model.conn_edges_on_port(p.id).next().is_none()
        {
            out.push(Finding::DelegatedPortWithoutConnections {
                port: model.port_path(p.id),
            });
        }
    }

    // Views with no edges. Stdlib (preset) views are substrate, not model
    // intent, and are not reported.
    for v in model.views.values() {
        if model.is_stdlib(v.id.raw()) || filter.is_some_and(|f| !f.contains(&v.id)) {
            continue;
        }
        if !model.edges.values().any(|e| e.views.contains(&v.id)) {
            out.push(Finding::EmptyView {
                view: v.name.clone(),
            });
        }
    }

    // Types with no instances. The stdlib (preset) is substrate, not model
    // intent, so it is not reported.
    for rt in model.rels.values() {
        if model.is_stdlib(rt.id.raw()) {
            continue;
        }
        let used = model
            .edges
            .values()
            .any(|e| matches!(&e.payload, EdgePayload::Rel { rel, .. } if *rel == rt.id));
        if !used {
            out.push(Finding::TypeWithoutInstances {
                type_kind: "rel",
                name: rt.name.clone(),
            });
        }
    }
    for ct in model.conns.values() {
        if model.is_stdlib(ct.id.raw()) {
            continue;
        }
        let used = model
            .edges
            .values()
            .any(|e| matches!(&e.payload, EdgePayload::Conn { conn, .. } if *conn == ct.id));
        if !used {
            out.push(Finding::TypeWithoutInstances {
                type_kind: "conn",
                name: ct.name.clone(),
            });
        }
    }

    out
}
