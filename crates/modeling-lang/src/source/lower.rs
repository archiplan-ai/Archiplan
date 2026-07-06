//! Lowering: a resolved project → one deterministic statement batch, plus a
//! span table mapping every statement back to the source item it came from.
//!
//! Emission order is independent of authoring order:
//!
//! 1. nodes, parents before children (path order), each carrying its
//!    declared ports;
//! 2. views (name order);
//! 3. rel types, topologically by pattern references between them (name
//!    tie-break; a reference cycle is `E_DEF_CYCLE`);
//! 4. conn types (name order — they reference only nodes and rels);
//! 5. rel edges, grouped by type in the same topological order — classifier
//!    edges land before the shapes that consult them — authoring order
//!    within a group;
//! 6. conn edges, authoring order;
//! 7. applications, authoring order — an application needs its outer port
//!    attached, so delegation chains read outward-in.

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
            }),
            c.span,
            &mut batch,
            &mut spans,
        );
    }

    // 5. Rel edges: preset types first (rank 0), then user types in
    // topological order; authoring order within a type.
    let mut rel_edges: Vec<(usize, usize, &EdgeR)> = Vec::new();
    let mut conn_edges: Vec<&EdgeR> = Vec::new();
    for (seq, e) in res.edges.iter().enumerate() {
        match e {
            EdgeR::Rel { rel, .. } => {
                let rank = rel_rank.get(rel.as_str()).copied().unwrap_or(0);
                rel_edges.push((rank, seq, e));
            }
            EdgeR::Conn { .. } => conn_edges.push(e),
        }
    }
    rel_edges.sort_by_key(|(rank, seq, _)| (*rank, *seq));
    for (_, _, e) in rel_edges {
        let EdgeR::Rel {
            rel,
            source,
            target,
            views,
            span,
        } = e
        else {
            unreachable!("filtered to rel edges");
        };
        push(
            Statement::RelEdge {
                rel: rel.clone(),
                source: source.clone(),
                target: target.clone(),
                views: views.clone(),
            },
            *span,
            &mut batch,
            &mut spans,
        );
    }

    // 6. Conn edges.
    for e in conn_edges {
        let EdgeR::Conn {
            conn,
            source,
            carrier,
            rev_carrier,
            target,
            views,
            span,
        } = e
        else {
            unreachable!("filtered to conn edges");
        };
        push(
            Statement::ConnEdge {
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
            },
            *span,
            &mut batch,
            &mut spans,
        );
    }

    // 7. Applications.
    for a in &res.apps {
        push(
            Statement::App {
                node: a.node.clone(),
                port: a.port.clone(),
                route: a.route.clone(),
                inner: End {
                    node: a.inner_node.clone(),
                    port: a.inner_port.clone(),
                },
            },
            a.span,
            &mut batch,
            &mut spans,
        );
    }

    Ok(Lowered { batch, spans })
}
