//! Addressing (`archi/requirements/element-addressing/`): every finding,
//! diagnostic and rendered report line carries the id of the element it
//! concerns — a node path, a port path, canonical edge surface text, a view or
//! type name, or a requirement/stressor/session slug. A reader who sees a
//! wrong or broken line names the element instead of paraphrasing the spec,
//! and the id resolves straight back — through `query`/`search`, or the shared
//! element resolver a link and a `satisfied-by` already speak — to the one
//! element it names. The ids already exist inside the tool; addressing is the
//! thin projection that carries them out to the surfaces a human reads.

use serde_json::Value;

use modeling_lang::Finding;

use crate::docs::DocFinding;

/// The id of the element a line concerns, tagged by kind so a reader knows how
/// it resolves back: a `node`/`port`/`edge` through the shared element
/// resolver (and, for a node, `query --scope`); a `view` through
/// `query --view`, an `edge-type` through `query --edge-type`; a
/// `requirement`/`stressor`/`session`/`decision` slug through `search`.
pub struct Address {
    /// The element id — the string a reader quotes back.
    pub id: String,
    /// What kind of element the id names, and thus how it resolves.
    pub kind: &'static str,
}

impl Address {
    fn of(id: impl Into<String>, kind: &'static str) -> Self {
        Address {
            id: id.into(),
            kind,
        }
    }

    /// Stamp the id and its kind onto a finding's JSON object, so the `--json`
    /// surface carries the same address the human line does.
    pub fn stamp(&self, value: &mut Value) {
        if let Some(o) = value.as_object_mut() {
            o.insert("id".to_string(), Value::String(self.id.clone()));
            o.insert("id_kind".to_string(), Value::String(self.kind.to_string()));
        }
    }
}

/// The element a model-completeness finding concerns.
pub fn of_finding(f: &Finding) -> Address {
    match f {
        // The connection edge whose carried traffic finds no route — named by
        // its canonical surface text, so a line about an edge names the edge.
        Finding::UnroutedTraffic { statement, .. } => Address::of(statement.pseudo(), "edge"),
        Finding::UnusedPort { port } => Address::of(port.clone(), "port"),
        Finding::EmptyView { view } => Address::of(view.clone(), "view"),
        Finding::TypeWithoutInstances { name, .. } => Address::of(name.clone(), "edge-type"),
    }
}

/// The doc primitive a doc-completeness finding concerns.
pub fn of_doc_finding(f: &DocFinding) -> Address {
    match f {
        DocFinding::UnsatisfiedRequirement { requirement }
        | DocFinding::DeferredRequirement { requirement, .. }
        | DocFinding::UnverifiedSatisfaction { requirement } => {
            Address::of(requirement.clone(), "requirement")
        }
        DocFinding::PendingStressor { stressor, .. }
        | DocFinding::BreakingUnanswered { stressor, .. }
        | DocFinding::AcceptedUnjustified { stressor, .. } => {
            Address::of(stressor.clone(), "stressor")
        }
        DocFinding::OffListAxis { decision, .. } => Address::of(decision.clone(), "decision"),
        DocFinding::EmptySession { session } | DocFinding::FoldedAwaitsRemint { session, .. } => {
            Address::of(session.clone(), "session")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use modeling_lang::Statement;

    #[test]
    fn a_finding_about_an_edge_carries_its_canonical_surface_text() {
        let edge = Statement::RelEdge {
            rel: "dep".to_string(),
            source: "A".to_string(),
            target: "Hub".to_string(),
            views: Vec::new(),
        };
        let f = Finding::UnroutedTraffic {
            statement: Box::new(edge),
            port: "A.out".to_string(),
        };
        let addr = of_finding(&f);
        assert_eq!(addr.id, "A dep Hub");
        assert_eq!(addr.kind, "edge");
    }

    #[test]
    fn model_findings_carry_the_precise_element_id() {
        assert_eq!(
            of_finding(&Finding::UnusedPort {
                port: "Auth.store".to_string()
            })
            .id,
            "Auth.store"
        );
        assert_eq!(
            of_finding(&Finding::EmptyView {
                view: "login_flow".to_string()
            })
            .kind,
            "view"
        );
        assert_eq!(
            of_finding(&Finding::TypeWithoutInstances {
                type_kind: "conn",
                name: "wire".to_string()
            })
            .kind,
            "edge-type"
        );
    }

    #[test]
    fn doc_findings_carry_the_primitive_slug() {
        let r = of_doc_finding(&DocFinding::DeferredRequirement {
            requirement: "token-rotation".to_string(),
            reason: "postponed".to_string(),
        });
        assert_eq!(r.id, "token-rotation");
        assert_eq!(r.kind, "requirement");

        let s = of_doc_finding(&DocFinding::BreakingUnanswered {
            stressor: "credential-stuffing".to_string(),
            session: "auth-hardening".to_string(),
        });
        assert_eq!(s.id, "credential-stuffing");
        assert_eq!(s.kind, "stressor");
    }

    #[test]
    fn the_stamp_carries_the_id_into_json() {
        let mut v = serde_json::json!({ "kind": "unused_port", "port": "Auth.store" });
        of_finding(&Finding::UnusedPort {
            port: "Auth.store".to_string(),
        })
        .stamp(&mut v);
        assert_eq!(v["id"], "Auth.store");
        assert_eq!(v["id_kind"], "port");
    }
}
