//! Read statements: `ports`, `check` and `dump`.
//!
//! Results are the statement objects that would recreate the sliced part of
//! the model. Any read can be restricted to the edges of one or more views;
//! applications are untagged plumbing and belong to the views of the
//! connection edges they route.

use std::collections::BTreeSet;

use crate::ids::{EdgeId, NodeId, PortId, ViewId};
use crate::model::{Edge, EdgePayload, Model};
use crate::result::Finding;
use crate::statement::Statement;

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

/// Every statement that attaches to a port of the node: connection edges on
/// its ports and applications delegating them (on either side).
pub(crate) fn ports(
    model: &Model,
    node: NodeId,
    filter: Option<&BTreeSet<ViewId>>,
) -> Vec<Statement> {
    let mut out = Vec::new();
    for e in model.edges.values() {
        let attaches = match &e.payload {
            EdgePayload::Conn {
                src_port, dst_port, ..
            } => model.ports[src_port].node == node || model.ports[dst_port].node == node,
            EdgePayload::App { outer, inner, .. } => {
                model.ports[outer].node == node || model.ports[inner].node == node
            }
            EdgePayload::Rel { .. } => false,
        };
        if attaches && edge_in_filter(model, e, filter) {
            out.push(model.edge_statement(e));
        }
    }
    out
}

/// The model (or a view slice of it) rendered as replayable statements, in
/// creation order. Stdlib declarations are omitted: every model already has
/// them.
pub(crate) fn dump(model: &Model, filter: Option<&BTreeSet<ViewId>>) -> Vec<Statement> {
    let mut items: Vec<(u64, Statement)> = Vec::new();

    let included_edges: Vec<&Edge> = model
        .edges
        .values()
        .filter(|e| edge_in_filter(model, e, filter))
        .collect();

    match filter {
        None => {
            for v in model.views.values() {
                items.push((v.id.raw(), model.view_statement(v)));
            }
            for r in model.rels.values() {
                if !r.stdlib {
                    items.push((r.id.raw(), model.rel_statement(r)));
                }
            }
            for c in model.conns.values() {
                items.push((c.id.raw(), model.conn_statement(c)));
            }
            for n in model.nodes.values() {
                items.push((n.id.raw(), model.node_statement(n.id)));
            }
        }
        Some(f) => {
            // Minimal closure that lets the slice parse: the filter views, the
            // types of included edges, and the involved nodes with their
            // ancestors. Classifier edges outside the slice are not pulled in,
            // so replaying a slice can hit shape findings — inherent to slicing.
            let mut nodes: BTreeSet<NodeId> = BTreeSet::new();
            let include_node = |set: &mut BTreeSet<NodeId>, n: NodeId| {
                for a in model.scope_chain(n) {
                    set.insert(a);
                }
            };
            for e in &included_edges {
                match &e.payload {
                    EdgePayload::Rel { src, dst, rel } => {
                        include_node(&mut nodes, *src);
                        include_node(&mut nodes, *dst);
                        let rt = &model.rels[rel];
                        if !rt.stdlib {
                            items.push((rt.id.raw(), model.rel_statement(rt)));
                        }
                    }
                    EdgePayload::Conn {
                        src_port,
                        carrier,
                        dst_port,
                        conn,
                    } => {
                        include_node(&mut nodes, model.ports[src_port].node);
                        include_node(&mut nodes, model.ports[dst_port].node);
                        if let Some(c) = carrier {
                            include_node(&mut nodes, *c);
                        }
                        let ct = &model.conns[conn];
                        items.push((ct.id.raw(), model.conn_statement(ct)));
                    }
                    EdgePayload::App { outer, inner, .. } => {
                        include_node(&mut nodes, model.ports[outer].node);
                        include_node(&mut nodes, model.ports[inner].node);
                    }
                }
            }
            for v in f {
                let vd = &model.views[v];
                items.push((vd.id.raw(), model.view_statement(vd)));
            }
            for n in nodes {
                items.push((n.raw(), model.node_statement(n)));
            }
            items.sort_by_key(|(id, _)| *id);
            items.dedup();
        }
    }

    for e in included_edges {
        items.push((e.id.raw(), model.edge_statement(e)));
    }
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

    // Views with no edges.
    for v in model.views.values() {
        if filter.is_some_and(|f| !f.contains(&v.id)) {
            continue;
        }
        if !model.edges.values().any(|e| e.views.contains(&v.id)) {
            out.push(Finding::EmptyView {
                view: v.name.clone(),
            });
        }
    }

    // Types with no instances. The stdlib is substrate, not model intent, so
    // it is not reported.
    for rt in model.rels.values() {
        if rt.stdlib {
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
