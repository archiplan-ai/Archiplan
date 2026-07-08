//! Lowering: a resolved project → one deterministic statement batch, plus a
//! span table mapping every statement back to the source item it came from.
//!
//! Emission order is a function of the model alone — module names, file
//! splits and authoring order never move a statement:
//!
//! 1. nodes, parents before children (path order), each carrying its
//!    declared ports;
//! 2. views (name order);
//! 3. rel types, topologically by pattern references between them (name
//!    tie-break; a reference cycle is `E_DEF_CYCLE`);
//! 4. conn types (name order — they reference only nodes and rels);
//! 5. rel edges, grouped by type in the same topological order — classifier
//!    edges land before the shapes that consult them — canonical surface
//!    order within a group;
//! 6. conn edges, canonical surface order;
//! 7. applications, delegation-chain order — an application needs its outer
//!    port attached, so the application that attaches it lowers first and
//!    chains read outward-in — canonical surface order among the ready.

use std::collections::{BTreeMap, BTreeSet};

use crate::statement::{Definition, End, PatternExpr, Statement};

use super::resolve::{EdgeR, Resolution};
use super::span::{Diagnostic, Span};

/// The batch and its index-aligned source spans.
pub(crate) struct Lowered {
    pub batch: Vec<Statement>,
    pub spans: Vec<Span>,
}

/// User rel names referenced by a pattern.
fn pattern_rel<'a>(p: &'a PatternExpr, user_rels: &BTreeSet<&str>) -> Option<&'a str> {
    match p {
        PatternExpr::Classified { rel, .. } if user_rels.contains(rel.as_str()) => Some(rel),
        _ => None,
    }
}

/// Topological order of the user rel types under pattern references, name
/// tie-broken; `Err` carries the participants of a reference cycle.
fn rel_topo(res: &Resolution) -> Result<Vec<String>, Vec<String>> {
    let user_rels: BTreeSet<&str> = res.rels.iter().map(|r| r.name.as_str()).collect();
    // name → names it depends on
    let mut deps: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for r in &res.rels {
        let entry = deps.entry(r.name.as_str()).or_default();
        for p in [&r.source, &r.target] {
            if let Some(dep) = pattern_rel(p, &user_rels)
                && dep != r.name
            {
                entry.insert(dep);
            }
        }
    }
    let mut order = Vec::new();
    let mut placed: BTreeSet<&str> = BTreeSet::new();
    while placed.len() < deps.len() {
        let mut ready: Vec<&str> = deps
            .iter()
            .filter(|(name, ds)| !placed.contains(*name) && ds.iter().all(|d| placed.contains(d)))
            .map(|(name, _)| *name)
            .collect();
        if ready.is_empty() {
            return Err(deps
                .keys()
                .filter(|n| !placed.contains(*n))
                .map(|n| n.to_string())
                .collect());
        }
        ready.sort_unstable();
        for name in ready {
            placed.insert(name);
            order.push(name.to_string());
        }
    }
    Ok(order)
}

/// Lower a resolution to the statement batch.
pub(crate) fn lower(res: &Resolution) -> Result<Lowered, Vec<Diagnostic>> {
    let mut batch = Vec::new();
    let mut spans = Vec::new();
    let push = |stmt: Statement, span: Span, batch: &mut Vec<Statement>, spans: &mut Vec<Span>| {
        batch.push(stmt);
        spans.push(span);
    };

    // 1. Nodes: lexicographic path order puts every parent before its
    // children (a parent is a strict prefix of its child's path).
    for (path, info) in &res.nodes {
        push(
            Statement::Define(Definition::Node {
                path: path.clone(),
                ports: (!info.ports.is_empty()).then(|| info.ports.clone()),
                doc: info.doc.clone(),
                port_docs: (!info.port_docs.is_empty()).then(|| info.port_docs.clone()),
            }),
            info.span,
            &mut batch,
            &mut spans,
        );
    }

    // 2. Views.
    let mut views = res.views.clone();
    views.sort_by(|a, b| a.name.cmp(&b.name));
    for v in views {
        push(
            Statement::Define(Definition::View {
                name: v.name.clone(),
                doc: v.doc.clone(),
            }),
            v.span,
            &mut batch,
            &mut spans,
        );
    }

    // 3. Rel types, topologically.
    let rel_order = rel_topo(res).map_err(|cycle| {
        cycle
            .iter()
            .filter_map(|name| res.rels.iter().find(|r| &r.name == name))
            .map(|r| {
                Diagnostic::new(
                    "E_DEF_CYCLE",
                    format!(
                        "the shape of `{}` participates in a reference cycle between relation types",
                        r.name
                    ),
                    r.span,
                )
            })
            .collect::<Vec<_>>()
    })?;
    let rel_rank: BTreeMap<&str, usize> = rel_order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i + 1))
        .collect();
    let rels_by_name: BTreeMap<&str, &super::resolve::RelDefR> =
        res.rels.iter().map(|r| (r.name.as_str(), r)).collect();
    for name in &rel_order {
        let r = rels_by_name[name.as_str()];
        push(
            Statement::Define(Definition::Rel {
                name: r.name.clone(),
                trans: r.trans,
                directed: r.directed,
                source: r.source.clone(),
                target: r.target.clone(),
                doc: r.doc.clone(),
            }),
            r.span,
            &mut batch,
            &mut spans,
        );
    }

    // 4. Conn types.
    let mut conns = res.conns.clone();
    conns.sort_by(|a, b| a.name.cmp(&b.name));
    for c in conns {
        push(
            Statement::Define(Definition::Conn {
                name: c.name.clone(),
                directed: c.directed,
                source: c.source.clone(),
                carrier: c.fwd_carrier.clone(),
                rev_carrier: c.rev_carrier.clone(),
                target: c.target.clone(),
                doc: c.doc.clone(),
            }),
            c.span,
            &mut batch,
            &mut spans,
        );
    }

    // 5. Rel edges: preset types first (rank 0), then user types in
    // topological order; canonical surface order within a type, so the
    // batch — and every render downstream — is a function of the model
    // alone, never of module names or authoring order.
    let mut rel_edges: Vec<(usize, String, Statement, Span)> = Vec::new();
    let mut conn_edges: Vec<(String, Statement, Span)> = Vec::new();
    for e in &res.edges {
        match e {
            EdgeR::Rel {
                rel,
                source,
                target,
                views,
                span,
            } => {
                let rank = rel_rank.get(rel.as_str()).copied().unwrap_or(0);
                let stmt = Statement::RelEdge {
                    rel: rel.clone(),
                    source: source.clone(),
                    target: target.clone(),
                    views: views.clone(),
                };
                rel_edges.push((rank, stmt.pseudo(), stmt, *span));
            }
            EdgeR::Conn {
                conn,
                source,
                carrier,
                rev_carrier,
                target,
                views,
                span,
            } => {
                let stmt = Statement::ConnEdge {
                    conn: conn.clone(),
                    source: End {
                        node: source.0.clone(),
                        port: source.1.clone(),
                    },
                    carrier: carrier.clone(),
                    rev_carrier: rev_carrier.clone(),
                    target: End {
                        node: target.0.clone(),
                        port: target.1.clone(),
                    },
                    views: views.clone(),
                };
                conn_edges.push((stmt.pseudo(), stmt, *span));
            }
        }
    }
    rel_edges.sort_by(|(ra, ka, ..), (rb, kb, ..)| ra.cmp(rb).then_with(|| ka.cmp(kb)));
    for (_, _, stmt, span) in rel_edges {
        push(stmt, span, &mut batch, &mut spans);
    }

    // 6. Conn edges, canonical surface order.
    conn_edges.sort_by(|(ka, ..), (kb, ..)| ka.cmp(kb));
    for (_, stmt, span) in conn_edges {
        push(stmt, span, &mut batch, &mut spans);
    }

    // 7. Applications: delegation-chain order — an application attaches
    // its inner port, and a chained application needs its outer port
    // attached first — canonical surface order among the ready. Authoring
    // order stops being load-bearing: a chain authored inner-module-first
    // lowers outward-in all the same. Anything unplaced (a cycle cannot
    // compile) keeps authoring order so diagnostics read as written.
    let apps: Vec<(String, Statement, Span)> = res
        .apps
        .iter()
        .map(|a| {
            let stmt = Statement::App {
                node: a.node.clone(),
                port: a.port.clone(),
                route: a.route.clone(),
                inner: End {
                    node: a.inner_node.clone(),
                    port: a.inner_port.clone(),
                },
            };
            (stmt.pseudo(), stmt, a.span)
        })
        .collect();
    let attaches: Vec<(String, String)> = res
        .apps
        .iter()
        .map(|a| (format!("{}.{}", a.node, a.inner_node), a.inner_port.clone()))
        .collect();
    let deps: Vec<Vec<usize>> = res
        .apps
        .iter()
        .map(|a| {
            attaches
                .iter()
                .enumerate()
                .filter(|(_, (path, port))| *path == a.node && *port == a.port)
                .map(|(i, _)| i)
                .collect()
        })
        .collect();
    let mut placed = vec![false; apps.len()];
    let mut remaining = apps.len();
    while remaining > 0 {
        let mut ready: Vec<usize> = (0..apps.len())
            .filter(|&i| !placed[i] && deps[i].iter().all(|&d| placed[d]))
            .collect();
        if ready.is_empty() {
            break;
        }
        ready.sort_by(|&a, &b| apps[a].0.cmp(&apps[b].0));
        for i in ready {
            placed[i] = true;
            remaining -= 1;
            push(apps[i].1.clone(), apps[i].2, &mut batch, &mut spans);
        }
    }
    for (i, (_, stmt, span)) in apps.iter().enumerate() {
        if !placed[i] {
            push(stmt.clone(), *span, &mut batch, &mut spans);
        }
    }

    Ok(Lowered { batch, spans })
}
