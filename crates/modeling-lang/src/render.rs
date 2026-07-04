//! Rendering stored elements back into the statements that recreate them.
//!
//! All node references render as absolute paths from the root, and statements
//! that only make sense inside a scope (`node` in a nested scope, applications)
//! are wrapped in a `Parent { ... }` block, so every rendered statement replays
//! from the root scope.

use crate::ids::{NodeId, ViewId};
use crate::model::{ConnType, Edge, EdgePayload, Model, Pattern, RelType, ViewDef};

use std::collections::BTreeSet;

impl Model {
    pub(crate) fn render_pattern(&self, p: &Pattern) -> String {
        match p {
            Pattern::Any => "*".to_string(),
            Pattern::Exact(n) => format!("({})", self.node_path(*n)),
            Pattern::Classified { anchor, rel } => {
                format!("({} {} *)", self.node_path(*anchor), self.rels[rel].name)
            }
        }
    }

    pub(crate) fn render_rel_decl(&self, rt: &RelType) -> String {
        let arrow = if rt.directed { "->" } else { "<->" };
        let trans = if rt.trans { "trans " } else { "" };
        format!(
            "rel {trans}{} := {} {arrow} {};",
            rt.name,
            self.render_pattern(&rt.src),
            self.render_pattern(&rt.dst)
        )
    }

    pub(crate) fn render_conn_decl(&self, ct: &ConnType) -> String {
        let arrow = if ct.directed { "->" } else { "<->" };
        match &ct.carrier {
            Some(c) => format!(
                "conn {} := {} {}{arrow} {};",
                ct.name,
                self.render_pattern(&ct.src),
                self.render_pattern(c),
                self.render_pattern(&ct.dst)
            ),
            None => format!(
                "conn {} := {} {arrow} {};",
                ct.name,
                self.render_pattern(&ct.src),
                self.render_pattern(&ct.dst)
            ),
        }
    }

    pub(crate) fn render_view_decl(&self, v: &ViewDef) -> String {
        format!("view {};", v.name)
    }

    /// `node X;` at the root, `Parent.Path { node X; }` in a nested scope.
    pub(crate) fn render_node_stmt(&self, node: NodeId) -> String {
        let n = &self.nodes[&node];
        match n.parent {
            None => format!("node {};", n.name),
            Some(p) => format!("{} {{ node {}; }}", self.node_path(p), n.name),
        }
    }

    fn render_views_suffix(&self, views: &BTreeSet<ViewId>) -> String {
        if views.is_empty() {
            return String::new();
        }
        let names: Vec<&str> = views.iter().map(|v| self.views[v].name.as_str()).collect();
        format!(" in {}", names.join(", "))
    }

    pub(crate) fn render_edge(&self, e: &Edge) -> String {
        match &e.payload {
            EdgePayload::Rel { rel, src, dst } => {
                format!(
                    "{} {} {}{};",
                    self.node_path(*src),
                    self.rels[rel].name,
                    self.node_path(*dst),
                    self.render_views_suffix(&e.views)
                )
            }
            EdgePayload::Conn {
                conn,
                src_port,
                carrier,
                dst_port,
            } => {
                let sp = &self.ports[src_port];
                let dp = &self.ports[dst_port];
                let carrier = match carrier {
                    Some(c) => format!("({})", self.node_path(*c)),
                    None => String::new(),
                };
                format!(
                    "{}({}) {}{carrier} {}({}){};",
                    self.node_path(sp.node),
                    sp.name,
                    self.conns[conn].name,
                    self.node_path(dp.node),
                    dp.name,
                    self.render_views_suffix(&e.views)
                )
            }
            EdgePayload::App {
                outer,
                qualifier,
                inner,
            } => {
                let op = &self.ports[outer];
                let ip = &self.ports[inner];
                let qual = match qualifier {
                    Some(q) => {
                        let inner_txt = match q {
                            Pattern::Any => "*".to_string(),
                            Pattern::Exact(n) => self.node_path(*n),
                            Pattern::Classified { anchor, rel } => {
                                format!("{} {} *", self.node_path(*anchor), self.rels[rel].name)
                            }
                        };
                        format!("({inner_txt})")
                    }
                    None => String::new(),
                };
                format!(
                    "{} {{ {}{qual} = {}({}); }}",
                    self.node_path(op.node),
                    op.name,
                    self.nodes[&ip.node].name,
                    ip.name
                )
            }
        }
    }
}
