//! Restatement/noop semantics, views, routing, deletion cascades, port
//! lifecycle, rename reference-safety, transitivity and scopes.

mod common;

use common::*;
use modeling_lang::{ErrorCode, Finding, Session};

const WORKED_EXAMPLE: &str = r#"
rel trans of_sort := * -> *;
node Functional;
node Data;
node Service;
Service of_sort Functional;
node Payments;
node Orders;
Service type_of Payments;
Service type_of Orders;
node OrderId;
OrderId of_sort Data;
conn confirm := (Service type_of *) (OrderId)-> (Service type_of *);
Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);
node Orders {
  node ConfirmationHandler;
  handle_confirmation = ConfirmationHandler(handle_confirmation);
}
"#;

const ROUTING_EXAMPLE: &str = r#"
node Message; node OrderCreated; node PaymentFailed; node ShipmentDue;
Message type_of OrderCreated; Message type_of PaymentFailed; Message type_of ShipmentDue;
node Service; node Payments; node Shipping; node Orders;
Service type_of Payments; Service type_of Shipping; Service type_of Orders;
conn send := (Service type_of *) (Message type_of *)-> (Service type_of *);
Payments(payment_events)  send(PaymentFailed) Orders(events);
Shipping(shipping_events) send(OrderCreated) Orders(events);
node Orders {
  node OrderHandler; node PaymentHandler;
  events(OrderCreated)  = OrderHandler(handle);
  events(PaymentFailed) = PaymentHandler(handle);
}
"#;

// ---- restatement / noop -----------------------------------------------

#[test]
fn identical_restatements_are_noops() {
    let mut s = session_with("node A; view v; rel dep := * -> *; node B; A dep B;");
    assert!(is_noop(&outcome(&mut s, "node A;")));
    assert!(is_noop(&outcome(&mut s, "view v;")));
    assert!(is_noop(&outcome(&mut s, "rel dep := * -> *;")));
    assert!(is_noop(&outcome(&mut s, "A dep B;")));
}

#[test]
fn conn_edge_identity_is_type_ports_and_carrier() {
    let mut s = session_with(
        "node A; node B; node M; node N;
         conn send := * (*)-> *;
         A(out) send(M) B(recv);",
    );
    assert!(is_noop(&outcome(&mut s, "A(out) send(M) B(recv);")));
    // A different carrier through the same ports is a different edge.
    assert!(is_applied(&outcome(&mut s, "A(out) send(N) B(recv);")));
}

#[test]
fn undirected_edges_have_unordered_identity() {
    let mut s = session_with(
        "rel peers := * <-> *;
         conn link := * <-> *;
         node A; node B;
         A peers B;
         A(x) link B(y);",
    );
    assert!(is_noop(&outcome(&mut s, "B peers A;")));
    assert!(is_noop(&outcome(&mut s, "B(y) link A(x);")));
}

#[test]
fn several_edges_share_a_port() {
    let mut s = session_with(
        "node A; node B;
         conn c := * -> *;
         A(out) c B(in1);
         A(out) c B(in2);",
    );
    assert_eq!(
        statements(&mut s, "ports A"),
        vec!["A(out) c B(in1);", "A(out) c B(in2);"]
    );
}

#[test]
fn restating_an_application_is_a_noop() {
    let mut s = session_with(WORKED_EXAMPLE);
    let results = outcomes(
        &mut s,
        "Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }",
    );
    let modeling_lang::Outcome::Block(inner) = &results[0] else {
        panic!("expected block");
    };
    assert!(is_noop(&inner[0].outcome));
}

// ---- views ---------------------------------------------------------------

#[test]
fn views_extend_by_restatement_and_shrink_by_untag() {
    let mut s = session_with(
        "view flow; view fault;
         node A; node B; rel dep := * -> *;
         A dep B in flow;",
    );
    assert!(
        is_applied(&outcome(&mut s, "A dep B in fault;")),
        "restating with `in` extends"
    );
    assert!(is_noop(&outcome(&mut s, "A dep B in fault;")));
    assert!(
        is_noop(&outcome(&mut s, "A dep B;")),
        "restating without `in` is a noop"
    );
    assert!(statements(&mut s, "dump in flow").contains(&"A dep B in flow, fault;".to_string()));

    assert!(is_applied(&outcome(&mut s, "untag A dep B in flow;")));
    assert!(is_noop(&outcome(&mut s, "untag A dep B in flow;")));
    // An untagged edge belongs to no view and is invisible to filtered queries.
    assert!(is_applied(&outcome(&mut s, "untag A dep B in fault;")));
    assert!(
        !statements(&mut s, "dump in flow")
            .iter()
            .any(|l| l.contains("dep"))
    );
    assert!(statements(&mut s, "dump").contains(&"A dep B;".to_string()));
}

#[test]
fn deleting_a_view_only_drops_tags() {
    let mut s = session_with("view flow; node A; node B; rel dep := * -> *; A dep B in flow;");
    assert_eq!(cascade(&mut s, "delete view flow;"), vec!["view flow;"]);
    assert!(statements(&mut s, "dump").contains(&"A dep B;".to_string()));
    assert_eq!(err_code(&mut s, "dump in flow"), ErrorCode::UnknownName);
}

#[test]
fn applications_belong_to_the_views_of_the_edges_they_route() {
    let mut s = session_with(WORKED_EXAMPLE);
    outcomes(&mut s, "view flow; view other;");
    outcomes(
        &mut s,
        "Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation) in flow;",
    );
    assert_eq!(
        statements(&mut s, "ports Orders in flow"),
        vec![
            "Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation) in flow;",
            "Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }",
        ]
    );
    assert_eq!(
        statements(&mut s, "ports Orders in other"),
        Vec::<String>::new()
    );
}

// ---- routing by carried node -------------------------------------------

#[test]
fn qualified_delegations_route_by_carrier() {
    let mut s = session_with(ROUTING_EXAMPLE);
    assert_eq!(findings(&mut s, "check"), vec![]);
    // Traffic whose carrier matches no qualifier and has no unqualified
    // fallback is a finding, not an error.
    outcomes(
        &mut s,
        "Shipping(shipping_events) send(ShipmentDue) Orders(events);",
    );
    let f = findings(&mut s, "check");
    assert!(
        f.iter().any(|f| matches!(
            f,
            Finding::UnroutedTraffic { port, .. } if port == "Orders.events"
        )),
        "expected unrouted traffic, got {f:?}"
    );
    // An unqualified delegation catches what no qualifier matches.
    outcomes(&mut s, "Orders { node Fallback; events = Fallback(rest); }");
    assert_eq!(findings(&mut s, "check"), vec![]);
}

// ---- deletion ------------------------------------------------------------

#[test]
fn deleting_a_node_cascades_over_the_referencing_closure() {
    let mut s = session_with(WORKED_EXAMPLE);
    assert_eq!(
        cascade(&mut s, "delete Orders;"),
        vec![
            "node Orders;",
            "Service type_of Orders;",
            "Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);",
            "Orders { node ConfirmationHandler; }",
            "Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }",
        ]
    );
    // The last edge on Payments.send_confirmation went with the cascade, so
    // the port is gone too.
    assert_eq!(statements(&mut s, "ports Payments"), Vec::<String>::new());
}

#[test]
fn deleting_a_pattern_anchor_takes_the_type_and_its_edges() {
    let mut s = session_with(WORKED_EXAMPLE);
    let c = cascade(&mut s, "delete Service;");
    assert!(c.contains(&"node Service;".to_string()));
    assert!(c.contains(
        &"conn confirm := (Service type_of *) (OrderId)-> (Service type_of *);".to_string()
    ));
    assert!(c.contains(
        &"Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);".to_string()
    ));
    assert!(c.contains(
        &"Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }".to_string()
    ));
    assert_eq!(statements(&mut s, "ports Orders"), Vec::<String>::new());
    // Cascading away the last attached edge freed the port name: a fresh
    // first use may bind a new type.
    outcomes(&mut s, "conn d := * -> *; node Z;");
    assert!(is_applied(&outcome(
        &mut s,
        "Orders(handle_confirmation) d Z(z);"
    )));
}

#[test]
fn deleting_a_carrier_node_takes_the_types_whose_patterns_name_it() {
    let mut s = session_with(
        "node M; node A; node B;
         conn send := * (M)-> *;
         A(x) send(M) B(y);",
    );
    assert_eq!(
        cascade(&mut s, "delete M;"),
        vec!["node M;", "conn send := * (M)-> *;", "A(x) send(M) B(y);"]
    );
}

#[test]
fn deleting_a_rel_type_takes_types_whose_patterns_use_it() {
    let mut s = session_with(
        "rel r := * -> *;
         node A; node B; node D;
         A r B;
         rel needs := (A r *) -> *;
         B needs D;",
    );
    assert_eq!(
        cascade(&mut s, "delete rel r;"),
        vec![
            "rel r := * -> *;",
            "A r B;",
            "rel needs := (A r *) -> *;",
            "B needs D;"
        ]
    );
}

#[test]
fn deleting_a_conn_type_takes_its_ports_and_applications() {
    let mut s = session_with(
        "node A; node B;
         conn c := * -> *;
         A(p) c B(q);
         B { node I; q = I(r); }",
    );
    assert_eq!(
        cascade(&mut s, "delete conn c;"),
        vec!["conn c := * -> *;", "A(p) c B(q);", "B { q = I(r); }"]
    );
    // All ports of the deleted type are gone; the names are free for a new type.
    outcomes(&mut s, "conn d := * -> *;");
    assert!(is_applied(&outcome(&mut s, "A(p) d B(q);")));
}

#[test]
fn deleting_a_classifier_edge_is_soft_drift_not_cascade() {
    let mut s = session_with(WORKED_EXAMPLE);
    assert_eq!(
        cascade(&mut s, "delete Service type_of Payments;"),
        vec!["Service type_of Payments;"]
    );
    // The nonconforming connection edge remains, surfaced as a finding.
    assert!(statements(&mut s, "dump").contains(
        &"Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);".to_string()
    ));
    let f = findings(&mut s, "check");
    assert!(
        f.iter().any(|f| matches!(
            f,
            Finding::ShapeDrift { slot, actual, .. } if slot == "source" && actual == "Payments"
        )),
        "expected shape drift, got {f:?}"
    );
}

#[test]
fn deleting_the_last_connection_leaves_a_delegated_port_as_a_finding() {
    let mut s = session_with(WORKED_EXAMPLE);
    assert_eq!(
        cascade(
            &mut s,
            "delete Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);"
        ),
        vec!["Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);"]
    );
    // The delegated port survives through the application — legal but suspect.
    let f = findings(&mut s, "check");
    assert!(
        f.iter().any(|f| matches!(
            f,
            Finding::DelegatedPortWithoutConnections { port } if port == "Orders.handle_confirmation"
        )),
        "expected delegated-port finding, got {f:?}"
    );
    // Payments.send_confirmation had no application, so it is gone and free.
    outcomes(&mut s, "conn d := * -> *; node Z;");
    assert!(is_applied(&outcome(
        &mut s,
        "Payments(send_confirmation) d Z(z);"
    )));
}

#[test]
fn deleting_a_node_named_by_a_qualifier_takes_the_delegation() {
    let mut s = session_with(ROUTING_EXAMPLE);
    let c = cascade(&mut s, "delete OrderCreated;");
    assert!(c.contains(&"node OrderCreated;".to_string()));
    assert!(
        c.contains(&"Shipping(shipping_events) send(OrderCreated) Orders(events);".to_string())
    );
    assert!(c.contains(&"Orders { events(OrderCreated) = OrderHandler(handle); }".to_string()));
    // The unrelated qualified delegation stays.
    assert!(
        statements(&mut s, "dump")
            .contains(&"Orders { events(PaymentFailed) = PaymentHandler(handle); }".to_string())
    );
}

// ---- rename ---------------------------------------------------------------

#[test]
fn rename_is_reference_safe() {
    let mut s = session_with(WORKED_EXAMPLE);
    assert!(is_applied(&outcome(&mut s, "rename Payments PaySvc;")));
    assert!(is_applied(&outcome(&mut s, "rename Service Kind;")));
    let dump = statements(&mut s, "dump").join("\n");
    assert!(
        dump.contains("PaySvc(send_confirmation) confirm(OrderId) Orders(handle_confirmation);")
    );
    assert!(dump.contains("conn confirm := (Kind type_of *) (OrderId)-> (Kind type_of *);"));
    assert!(!dump.contains("Payments"));
    assert!(!dump.contains("Service"));
}

// ---- patterns and transitivity --------------------------------------------

#[test]
fn patterns_follow_the_transitive_closure() {
    let mut s = session_with(
        "node Service; node RestService; node Payments;
         Service type_of RestService;
         RestService type_of Payments;
         conn calls := (Service type_of *) -> (Service type_of *);
         node Api; Service type_of Api;",
    );
    // Payments is a Service only through RestService — a virtual pair.
    assert!(is_applied(&outcome(
        &mut s,
        "Api(out) calls Payments(recv);"
    )));
}

#[test]
fn non_transitive_relations_match_single_steps_only() {
    let mut s = session_with(
        "rel r := * -> *;
         node A; node B; node C; node D;
         A r B; B r C;
         rel needs := (A r *) -> *;",
    );
    assert!(is_applied(&outcome(&mut s, "B needs D;")));
    assert_eq!(err_code(&mut s, "C needs D;"), ErrorCode::ShapeViolation);
}

// ---- scopes -----------------------------------------------------------------

#[test]
fn open_resolves_lexically_and_survives_deletes() {
    let mut s = session_with("node Orders { node Handler; } node Payments;");
    outcomes(&mut s, "open Orders");
    assert_eq!(s.scope_path(), "Orders");
    outcomes(&mut s, "open Handler");
    assert_eq!(s.scope_path(), "Orders.Handler");
    // A name not found inward resolves outward, up to the root.
    outcomes(&mut s, "open Payments");
    assert_eq!(s.scope_path(), "Payments");

    outcomes(&mut s, "open Orders");
    cascade(&mut s, "delete Orders;");
    assert_eq!(
        s.scope_path(),
        "",
        "deleting the scope you stand in pops to the root"
    );
    assert!(is_applied(&outcome(&mut s, "node After;")));
    assert!(statements(&mut s, "dump").contains(&"node After;".to_string()));
}

#[test]
fn same_name_in_different_scopes_is_addressed_by_path() {
    let mut s = session_with(
        "node A { node Worker; }
         node B { node Worker; }
         rel dep := * -> *;
         A.Worker dep B.Worker;",
    );
    assert!(statements(&mut s, "dump").contains(&"A.Worker dep B.Worker;".to_string()));
}

// ---- findings ----------------------------------------------------------------

#[test]
fn empty_views_and_uninstantiated_types_are_findings() {
    let mut s = Session::new();
    assert_eq!(
        findings(&mut s, "check"),
        vec![],
        "the stdlib is not reported"
    );
    outcomes(&mut s, "view lonely; rel r := * -> *; conn c := * -> *;");
    let f = findings(&mut s, "check");
    assert!(f.contains(&Finding::EmptyView {
        view: "lonely".into()
    }));
    assert!(f.contains(&Finding::TypeWithoutInstances {
        kind: "rel",
        name: "r".into()
    }));
    assert!(f.contains(&Finding::TypeWithoutInstances {
        kind: "conn",
        name: "c".into()
    }));
}
