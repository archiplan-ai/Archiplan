//! Cascading deletion: reference integrity is hard.
//!
//! `delete` removes the seed element together with the full closure of
//! elements that reference it: scopes recursively, edges ending on or carrying
//! a doomed node, type declarations whose patterns name a doomed node or
//! relation (and, transitively, those types' edges and ports), and delegations
//! qualified by a doomed element. Shape conformance is deliberately *not* part
//! of the closure — deleting a classifier edge leaves dependent edges in place
//! and drifting, surfaced later as findings.
//!
//! The removed set is rendered as statements, in creation (id) order, before
//! anything is removed.

use std::collections::BTreeSet;

use crate::ids::{ConnId, EdgeId, NodeId, RelId, ViewId};
use crate::model::{EdgePayload, Model, Pattern};

pub(crate) enum Seed {
    Node(NodeId),
    Edge(EdgeId),
    Rel(RelId),
    Conn(ConnId),
    View(ViewId),
}

#[derive(Default)]
struct Doomed {
    nodes: BTreeSet<NodeId>,
    edges: BTreeSet<EdgeId>,
    rels: BTreeSet<RelId>,
    conns: BTreeSet<ConnId>,
    views: BTreeSet<ViewId>,
}

fn pattern_refs_doomed(p: &Pattern, d: &Doomed) -> bool {
    match p {
        Pattern::Any => false,
        Pattern::Exact(n) => d.nodes.contains(n),
        Pattern::Classified { anchor, rel } => d.nodes.contains(anchor) || d.rels.contains(rel),
    }
}

fn closure(model: &Model, seed: Seed) -> Doomed {
    let mut d = Doomed::default();
    match seed {
        Seed::Node(n) => {
            d.nodes.insert(n);
        }
        Seed::Edge(e) => {
            d.edges.insert(e);
        }
        Seed::Rel(r) => {
            d.rels.insert(r);
        }
        Seed::Conn(c) => {
            d.conns.insert(c);
        }
        Seed::View(v) => {
            d.views.insert(v);
        }
    }
    loop {
        let mut changed = false;

        for id in d.nodes.clone() {
            for &child in model.nodes[&id].children.values() {
                changed |= d.nodes.insert(child);
            }
        }

        for e in model.edges.values() {
            if d.edges.contains(&e.id) {
                continue;
            }
            let doomed = match &e.payload {
                EdgePayload::Rel { rel, src, dst } => {
                    d.rels.contains(rel) || d.nodes.contains(src) || d.nodes.contains(dst)
                }
                EdgePayload::Conn {
                    conn,
                    src_port,
                    carrier,
                    dst_port,
                } => {
                    d.conns.contains(conn)
                        || d.nodes.contains(&model.ports[src_port].node)
                        || d.nodes.contains(&model.ports[dst_port].node)
                        || carrier.is_some_and(|c| d.nodes.contains(&c))
                }
                EdgePayload::App {
                    outer,
                    qualifier,
                    inner,
                } => {
                    let op = &model.ports[outer];
                    let ip = &model.ports[inner];
                    d.nodes.contains(&op.node)
                        || d.nodes.contains(&ip.node)
                        || d.conns.contains(&op.conn)
                        || d.conns.contains(&ip.conn)
                        || qualifier
                            .as_ref()
                            .is_some_and(|q| pattern_refs_doomed(q, &d))
                }
            };
            if doomed {
                d.edges.insert(e.id);
                changed = true;
            }
        }

        for rt in model.rels.values() {
            if d.rels.contains(&rt.id) {
                continue;
            }
            if pattern_refs_doomed(&rt.src, &d) || pattern_refs_doomed(&rt.dst, &d) {
                d.rels.insert(rt.id);
                changed = true;
            }
        }

        for ct in model.conns.values() {
            if d.conns.contains(&ct.id) {
                continue;
            }
            let hit = pattern_refs_doomed(&ct.src, &d)
                || pattern_refs_doomed(&ct.dst, &d)
                || ct
                    .carrier
                    .as_ref()
                    .is_some_and(|c| pattern_refs_doomed(c, &d));
            if hit {
                d.conns.insert(ct.id);
                changed = true;
            }
        }

        if !changed {
            return d;
        }
    }
}

/// Compute the closure, render it, remove it, then sweep unattached ports.
/// Returns the cascade report: everything removed, rendered as statements in
/// creation order.
pub(crate) fn delete(model: &mut Model, seed: Seed) -> Vec<String> {
    let d = closure(model, seed);

    let mut rendered: Vec<(u64, String)> = Vec::new();
    for v in &d.views {
        rendered.push((v.raw(), model.render_view_decl(&model.views[v])));
    }
    for r in &d.rels {
        rendered.push((r.raw(), model.render_rel_decl(&model.rels[r])));
    }
    for c in &d.conns {
        rendered.push((c.raw(), model.render_conn_decl(&model.conns[c])));
    }
    for n in &d.nodes {
        rendered.push((n.raw(), model.render_node_stmt(*n)));
    }
    for e in &d.edges {
        rendered.push((e.raw(), model.render_edge(&model.edges[e])));
    }
    rendered.sort_by_key(|(id, _)| *id);

    for e in &d.edges {
        model.edges.remove(e);
    }
    for r in &d.rels {
        let name = model.rels[r].name.clone();
        model.rels.remove(r);
        model.rel_names.remove(&name);
    }
    for c in &d.conns {
        let name = model.conns[c].name.clone();
        model.conns.remove(c);
        model.conn_names.remove(&name);
    }
    for n in &d.nodes {
        let node = model.nodes.remove(n).expect("doomed node exists");
        for port in node.ports.values() {
            model.ports.remove(port);
        }
        match node.parent {
            Some(p) if !d.nodes.contains(&p) => {
                model
                    .nodes
                    .get_mut(&p)
                    .expect("parent exists")
                    .children
                    .remove(&node.name);
            }
            Some(_) => {}
            None => {
                model.root.remove(&node.name);
            }
        }
    }
    for v in &d.views {
        let name = model.views[v].name.clone();
        model.views.remove(v);
        model.view_names.remove(&name);
        for e in model.edges.values_mut() {
            e.views.remove(v);
        }
    }

    // A port lives as long as some connection or application attaches to it;
    // cascading away the last attached edge removes the port and frees its name.
    let orphaned: Vec<_> = model
        .ports
        .values()
        .filter(|p| !model.port_attached(p.id))
        .map(|p| (p.id, p.node, p.name.clone()))
        .collect();
    for (pid, node, name) in orphaned {
        model.ports.remove(&pid);
        if let Some(n) = model.nodes.get_mut(&node) {
            n.ports.remove(&name);
        }
    }

    rendered.into_iter().map(|(_, s)| s).collect()
}
