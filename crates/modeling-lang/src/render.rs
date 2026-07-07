//! Rendering stored elements back into the statements that recreate them.
//!
//! All node references render as absolute paths from the root; creations
//! render as `define` statements, which are idempotent, so dumps replay
//! safely.

use crate::ids::NodeId;
use crate::model::{ConnType, Edge, EdgePayload, Model, Pattern, RelType, ViewDef};
use crate::statement::{Definition, End, PatternExpr, Statement};

impl Model {
    pub(crate) fn pattern_expr(&self, p: &Pattern) -> PatternExpr {
        match p {
            Pattern::Any => PatternExpr::Any,
            Pattern::Exact(n) => PatternExpr::Exact {
                node: self.node_path(*n),
            },
            Pattern::Classified { anchor, rel } => PatternExpr::Classified {
                anchor: self.node_path(*anchor),
                rel: self.rels[rel].name.clone(),
            },
        }
    }

    pub(crate) fn node_statement(&self, node: NodeId) -> Statement {
        let declared = self.declared_ports(node);
        Statement::Define(Definition::Node {
            path: self.node_path(node),
            ports: (!declared.is_empty()).then_some(declared),
        })
    }

    pub(crate) fn view_statement(&self, v: &ViewDef) -> Statement {
        Statement::Define(Definition::View {
            name: v.name.clone(),
        })
    }

    pub(crate) fn rel_statement(&self, rt: &RelType) -> Statement {
        Statement::Define(Definition::Rel {
            name: rt.name.clone(),
            trans: rt.trans,
            directed: rt.directed,
            source: self.pattern_expr(&rt.src),
            target: self.pattern_expr(&rt.dst),
        })
    }

    pub(crate) fn conn_statement(&self, ct: &ConnType) -> Statement {
        Statement::Define(Definition::Conn {
            name: ct.name.clone(),
            directed: ct.directed,
            source: self.pattern_expr(&ct.src),
            carrier: ct.carrier.as_ref().map(|c| self.pattern_expr(c)),
            rev_carrier: ct.rev_carrier.as_ref().map(|c| self.pattern_expr(c)),
            target: self.pattern_expr(&ct.dst),
        })
    }

    fn view_names_of(&self, e: &Edge) -> Vec<String> {
        e.views.iter().map(|v| self.views[v].name.clone()).collect()
    }

    pub(crate) fn edge_statement(&self, e: &Edge) -> Statement {
        match &e.payload {
            EdgePayload::Rel { rel, src, dst } => Statement::RelEdge {
                rel: self.rels[rel].name.clone(),
                source: self.node_path(*src),
                target: self.node_path(*dst),
                views: self.view_names_of(e),
            },
            EdgePayload::Conn {
                conn,
                src_port,
                carrier,
                rev_carrier,
                dst_port,
            } => {
                let sp = &self.ports[src_port];
                let dp = &self.ports[dst_port];
                Statement::ConnEdge {
                    conn: self.conns[conn].name.clone(),
                    source: End {
                        node: self.node_path(sp.node),
                        port: sp.name.clone(),
                    },
                    carrier: carrier.map(|c| self.node_path(c)),
                    rev_carrier: rev_carrier.map(|c| self.node_path(c)),
                    target: End {
                        node: self.node_path(dp.node),
                        port: dp.name.clone(),
                    },
                    views: self.view_names_of(e),
                }
            }
            EdgePayload::App {
                outer,
                qualifier,
                inner,
            } => {
                let op = &self.ports[outer];
                let ip = &self.ports[inner];
                Statement::App {
                    node: self.node_path(op.node),
                    port: op.name.clone(),
                    route: qualifier.as_ref().map(|q| self.pattern_expr(q)),
                    inner: End {
                        node: self.nodes[&ip.node].name.clone(),
                        port: ip.name.clone(),
                    },
                }
            }
        }
    }
}
