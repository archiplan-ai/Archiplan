//! Definitions, absolute addressing, views, routing, deletion cascades,
//! rename reference-safety, transitivity, and pseudo-syntax rendering.

mod common;

use common::*;
use modeling_lang::{EdgeKind, ErrorCode, Finding, Outcome, PatternExpr, Statement, Workspace};
use serde_json::{Value, json};

fn routing_example() -> Workspace {
    ws_with(json!([
        { "stmt": "define", "node": "Message" },
        { "stmt": "define", "node": "OrderCreated" },
        { "stmt": "define", "node": "PaymentFailed" },
        { "stmt": "define", "node": "ShipmentDue" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Message", "target": "OrderCreated" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Message", "target": "PaymentFailed" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Message", "target": "ShipmentDue" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Shipping" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.OrderHandler" },
        { "stmt": "define", "node": "Orders.PaymentHandler" },
        { "stmt": "define", "conn": "send", "directed": true,
          "source": "*", "carrier": { "anchor": "Message", "rel": "type_of" }, "target": "*" },
        { "stmt": "conn-edge", "conn": "send",
          "source": { "node": "Payments", "port": "payment_events" }, "carrier": "PaymentFailed",
          "target": { "node": "Orders", "port": "events" } },
        { "stmt": "conn-edge", "conn": "send",
          "source": { "node": "Shipping", "port": "shipping_events" }, "carrier": "OrderCreated",
          "target": { "node": "Orders", "port": "events" } },
        { "stmt": "app", "node": "Orders", "port": "events",
          "route": { "node": "OrderCreated" }, "inner": { "node": "OrderHandler", "port": "handle" } },
        { "stmt": "app", "node": "Orders", "port": "events",
          "route": { "node": "PaymentFailed" }, "inner": { "node": "PaymentHandler", "port": "handle" } }
    ]))
}

// ---- definitions ----------------------------------------------------------

#[test]
fn define_is_idempotent() {
    let mut ws = Workspace::new();
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "A" })
    )));
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "A" })
    )));
    // Identical restatements of typed definitions are no-ops too; a divergent
    // one is rejected, never silently applied.
    let rel =
        json!({ "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" });
    assert!(is_applied(&outcome(&mut ws, rel.clone())));
    assert!(is_noop(&outcome(&mut ws, rel)));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "rel": "dep", "directed": false, "source": "*", "target": "*" })
        ),
        ErrorCode::Redeclared
    );
    // Whole-batch replays are safe: every statement no-ops.
    let batch = json!([
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "view": "flow" },
        { "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B", "views": ["flow"] }
    ]);
    assert!(outcomes(&mut ws, batch.clone()).iter().all(is_applied));
    assert!(outcomes(&mut ws, batch).iter().all(is_noop));
}

#[test]
fn node_redefine_replaces_internals_keeping_external_wiring() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.ConfirmationHandler" },
        { "stmt": "define", "conn": "confirm", "directed": true, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "confirm",
          "source": { "node": "Payments", "port": "send" }, "target": { "node": "Orders", "port": "handle" } },
        { "stmt": "app", "node": "Orders", "port": "handle",
          "inner": { "node": "ConfirmationHandler", "port": "handle" } }
    ]));
    // One atomic batch: reset the internals, rebuild them differently.
    let results = outcomes(
        &mut ws,
        json!([
            { "stmt": "redefine", "node": "Orders" },
            { "stmt": "define", "node": "Orders.RefundHandler" },
            { "stmt": "app", "node": "Orders", "port": "handle",
              "inner": { "node": "RefundHandler", "port": "handle" } }
        ]),
    );
    match &results[0] {
        Outcome::Applied { cascade: Some(c) } => {
            let lines: Vec<String> = c.iter().map(Statement::pseudo).collect();
            assert!(lines.contains(&"def node Orders.ConfirmationHandler".to_string()));
            assert!(lines.contains(&"Orders.handle = ConfirmationHandler.handle".to_string()));
        }
        o => panic!("expected redefine cascade, got {o:?}"),
    }
    // The external connection survived: the node, its port and the edge kept
    // their identity.
    let lines = dump_pseudo(&ws);
    assert!(lines.contains(&"Payments.send confirm Orders.handle".to_string()));
    assert!(lines.contains(&"Orders.handle = RefundHandler.handle".to_string()));
    // Redefining an already-empty scope is a no-op.
    let results = outcomes(
        &mut ws,
        json!([{ "stmt": "redefine", "node": "Orders.RefundHandler" }]),
    );
    assert!(is_noop(&results[0]));
}

#[test]
fn type_redefine_replaces_shape_and_lets_edges_drift() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" },
        { "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B" }
    ]));
    // Narrow the shape: existing edges are not re-checked eagerly.
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "redefine", "rel": "dep", "directed": true,
                "source": { "anchor": "Service", "rel": "type_of" },
                "target": { "anchor": "Service", "rel": "type_of" } })
    )));
    let f = findings(&mut ws, json!({ "stmt": "check" }));
    assert!(
        f.iter()
            .any(|f| matches!(f, Finding::ShapeDrift { slot, actual, .. }
            if slot == "source" && actual == "A")),
        "expected drift after redefine, got {f:?}"
    );
    // New edges are validated against the new shape.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rel-edge", "rel": "dep", "source": "B", "target": "A" })
        ),
        ErrorCode::ShapeViolation
    );
}

// ---- addressing ------------------------------------------------------------

#[test]
fn references_are_absolute_only() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "A.Worker" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "B.Worker" },
        { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" }
    ]));
    // Same name in different scopes, addressed by path.
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "rel-edge", "rel": "dep", "source": "A.Worker", "target": "B.Worker" })
    )));
    // A bare name never resolves against some ambient scope.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rel-edge", "rel": "dep", "source": "Worker", "target": "B" })
        ),
        ErrorCode::UnknownName
    );
}

// ---- restatement and views ---------------------------------------------------

#[test]
fn edge_identity_is_structural() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "M" },
        { "stmt": "define", "node": "N" },
        { "stmt": "define", "rel": "peers", "directed": false, "source": "*", "target": "*" },
        { "stmt": "define", "conn": "send", "directed": true, "source": "*", "carrier": "*", "target": "*" },
        { "stmt": "rel-edge", "rel": "peers", "source": "A", "target": "B" },
        { "stmt": "conn-edge", "conn": "send",
          "source": { "node": "A", "port": "out" }, "carrier": "M", "target": { "node": "B", "port": "recv" } }
    ]));
    // Undirected identity is unordered.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "rel-edge", "rel": "peers", "source": "B", "target": "A" })
    )));
    // Same ports, same carrier: the same edge.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "send",
                "source": { "node": "A", "port": "out" }, "carrier": "M", "target": { "node": "B", "port": "recv" } })
    )));
    // A different carrier through the same ports is a different edge.
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "send",
                "source": { "node": "A", "port": "out" }, "carrier": "N", "target": { "node": "B", "port": "recv" } })
    )));
}

#[test]
fn views_extend_by_restatement_and_shrink_by_untag() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "view": "flow" },
        { "stmt": "define", "view": "fault" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" },
        { "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B", "views": ["flow"] }
    ]));
    let edge = json!({ "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B" });
    let tagged = json!({ "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B", "views": ["fault"] });
    assert!(
        is_applied(&outcome(&mut ws, tagged.clone())),
        "restating with views extends"
    );
    assert!(is_noop(&outcome(&mut ws, tagged)));
    assert!(
        is_noop(&outcome(&mut ws, edge.clone())),
        "restating without views is a noop"
    );

    let sliced = edge_values(&mut ws, json!({ "stmt": "query", "views": ["flow"] }));
    assert_eq!(
        sliced,
        vec![json!({ "kind": "relation", "type": "dep", "directed": true,
                     "source": "A", "target": "B", "views": ["flow", "fault"] })]
    );

    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "untag", "edge": edge, "views": ["flow", "fault"] })
    )));
    let edge = json!({ "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B" });
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "untag", "edge": edge, "views": ["flow"] })
    )));
    // An untagged edge is invisible to filtered queries, present in full ones.
    assert!(edge_values(&mut ws, json!({ "stmt": "query", "views": ["flow"] })).is_empty());
    assert!(
        edge_values(&mut ws, json!({ "stmt": "query" }))
            .iter()
            .any(|e| e["type"] == "dep")
    );
}

#[test]
fn deleting_a_view_only_drops_tags() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "view": "flow" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" },
        { "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "B", "views": ["flow"] }
    ]));
    assert_eq!(
        cascade(&mut ws, json!({ "stmt": "delete", "view": "flow" })),
        vec!["def view flow"]
    );
    assert!(dump_pseudo(&ws).contains(&"A dep B".to_string()));
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "views": ["flow"] })),
        ErrorCode::UnknownName
    );
}

#[test]
fn applications_belong_to_the_views_of_the_edges_they_route() {
    let mut ws = routing_example();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "view": "flow" },
            { "stmt": "define", "view": "other" },
            { "stmt": "conn-edge", "conn": "send",
              "source": { "node": "Shipping", "port": "shipping_events" }, "carrier": "OrderCreated",
              "target": { "node": "Orders", "port": "events" }, "views": ["flow"] }
        ]),
    );
    // The view admits its edges and the nodes related to them: attachments,
    // the carried node, and the application routing the tagged edge.
    assert_eq!(
        node_ids(&mut ws, json!({ "stmt": "query", "views": ["flow"] })),
        vec!["OrderCreated", "Shipping", "Orders", "Orders.OrderHandler"]
    );
    assert_eq!(
        edge_values(&mut ws, json!({ "stmt": "query", "views": ["flow"] })),
        vec![
            json!({ "kind": "connection", "type": "send", "directed": true,
                    "source": "Shipping", "source_port": "shipping_events",
                    "target": "Orders", "target_port": "events",
                    "carrier": "OrderCreated", "views": ["flow"] }),
            json!({ "kind": "application",
                    "source": "Orders", "source_port": "events",
                    "target": "Orders.OrderHandler", "target_port": "handle",
                    "route": { "node": "OrderCreated" }, "views": ["flow"] }),
        ]
    );
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query", "views": ["other"] }));
    assert!(nodes.is_empty() && edges.is_empty());
}

// ---- routing -------------------------------------------------------------

#[test]
fn qualified_delegations_route_by_carrier() {
    let mut ws = routing_example();
    assert_eq!(findings(&mut ws, json!({ "stmt": "check" })), vec![]);
    // Traffic whose carrier matches no qualifier and has no unqualified
    // fallback is a finding, not an error.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "conn-edge", "conn": "send",
              "source": { "node": "Shipping", "port": "shipping_events" }, "carrier": "ShipmentDue",
              "target": { "node": "Orders", "port": "events" } }
        ]),
    );
    let f = findings(&mut ws, json!({ "stmt": "check" }));
    assert!(
        f.iter()
            .any(|f| matches!(f, Finding::UnroutedTraffic { port, .. } if port == "Orders.events")),
        "expected unrouted traffic, got {f:?}"
    );
    // An unqualified delegation catches what no qualifier matches.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Orders.Fallback" },
            { "stmt": "app", "node": "Orders", "port": "events", "inner": { "node": "Fallback", "port": "rest" } }
        ]),
    );
    assert_eq!(findings(&mut ws, json!({ "stmt": "check" })), vec![]);
}

// ---- deletion --------------------------------------------------------------

#[test]
fn deleting_a_node_cascades_over_the_referencing_closure() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Orders" },
        { "stmt": "define", "node": "OrderId" },
        { "stmt": "define", "conn": "confirm", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "carrier": { "node": "OrderId" },
          "target": { "anchor": "Service", "rel": "type_of" } },
        { "stmt": "conn-edge", "conn": "confirm",
          "source": { "node": "Payments", "port": "send_confirmation" },
          "carrier": "OrderId",
          "target": { "node": "Orders", "port": "handle_confirmation" } },
        { "stmt": "define", "node": "Orders.ConfirmationHandler" },
        { "stmt": "app", "node": "Orders", "port": "handle_confirmation",
          "inner": { "node": "ConfirmationHandler", "port": "handle_confirmation" } }
    ]));
    assert_eq!(
        cascade(&mut ws, json!({ "stmt": "delete", "node": "Orders" })),
        vec![
            "def node Orders",
            "Service type_of Orders",
            "Payments.send_confirmation confirm(OrderId) Orders.handle_confirmation",
            "def node Orders.ConfirmationHandler",
            "Orders.handle_confirmation = ConfirmationHandler.handle_confirmation",
        ]
    );
    // The last edge on Payments.send_confirmation went with the cascade, so
    // the port is gone and its name is free for a new type.
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query" }));
    let payments = nodes.iter().find(|n| n.id == "Payments").expect("node");
    assert!(payments.ports.is_empty());
    assert!(edges.iter().all(
        |e| e.kind == EdgeKind::Relation || (e.source != "Payments" && e.target != "Payments")
    ));
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Z" },
            { "stmt": "define", "conn": "d", "directed": true, "source": "*", "target": "*" },
            { "stmt": "conn-edge", "conn": "d",
              "source": { "node": "Payments", "port": "send_confirmation" }, "target": { "node": "Z", "port": "z" } }
        ]),
    );
}

#[test]
fn deleting_a_pattern_anchor_takes_types_and_their_edges() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "A" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "B" },
        { "stmt": "define", "conn": "calls", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "target": { "anchor": "Service", "rel": "type_of" } },
        { "stmt": "conn-edge", "conn": "calls",
          "source": { "node": "A", "port": "out" }, "target": { "node": "B", "port": "recv" } }
    ]));
    let c = cascade(&mut ws, json!({ "stmt": "delete", "node": "Service" }));
    assert!(c.contains(&"def node Service".to_string()));
    assert!(
        c.contains(&"def conn calls := (Service type_of *) -> (Service type_of *)".to_string())
    );
    assert!(c.contains(&"A.out calls B.recv".to_string()));
}

#[test]
fn deleting_a_rel_type_takes_types_whose_patterns_use_it() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "rel": "r", "directed": true, "source": "*", "target": "*" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "D" },
        { "stmt": "rel-edge", "rel": "r", "source": "A", "target": "B" },
        { "stmt": "define", "rel": "needs", "directed": true,
          "source": { "anchor": "A", "rel": "r" }, "target": "*" },
        { "stmt": "rel-edge", "rel": "needs", "source": "B", "target": "D" }
    ]));
    assert_eq!(
        cascade(&mut ws, json!({ "stmt": "delete", "rel": "r" })),
        vec![
            "def rel r := * -> *",
            "A r B",
            "def rel needs := (A r *) -> *",
            "B needs D"
        ]
    );
}

#[test]
fn deleting_a_conn_type_takes_its_ports_and_applications() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "B.I" },
        { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "c",
          "source": { "node": "A", "port": "p" }, "target": { "node": "B", "port": "q" } },
        { "stmt": "app", "node": "B", "port": "q", "inner": { "node": "I", "port": "r" } }
    ]));
    assert_eq!(
        cascade(&mut ws, json!({ "stmt": "delete", "conn": "c" })),
        vec!["def conn c := * -> *", "A.p c B.q", "B.q = I.r"]
    );
    // All ports of the deleted type are gone; the names are free for a new type.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "conn": "d", "directed": true, "source": "*", "target": "*" },
            { "stmt": "conn-edge", "conn": "d",
              "source": { "node": "A", "port": "p" }, "target": { "node": "B", "port": "q" } }
        ]),
    );
}

#[test]
fn deleting_a_classifier_edge_is_soft_drift_not_cascade() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "A" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "B" },
        { "stmt": "define", "conn": "calls", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "target": { "anchor": "Service", "rel": "type_of" } },
        { "stmt": "conn-edge", "conn": "calls",
          "source": { "node": "A", "port": "out" }, "target": { "node": "B", "port": "recv" } }
    ]));
    assert_eq!(
        cascade(
            &mut ws,
            json!({ "stmt": "delete",
                    "edge": { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "A" } })
        ),
        vec!["Service type_of A"]
    );
    // The nonconforming connection edge remains, surfaced as a finding.
    assert!(dump_pseudo(&ws).contains(&"A.out calls B.recv".to_string()));
    let f = findings(&mut ws, json!({ "stmt": "check" }));
    assert!(
        f.iter().any(|f| matches!(f, Finding::ShapeDrift { slot, actual, expected, .. }
            if slot == "source" && actual == "A"
               && *expected == PatternExpr::Classified { anchor: "Service".into(), rel: "type_of".into() })),
        "expected shape drift, got {f:?}"
    );
}

#[test]
fn deleting_the_last_connection_leaves_a_delegated_port_as_a_finding() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.H" },
        { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "c",
          "source": { "node": "A", "port": "out" }, "target": { "node": "Orders", "port": "events" } },
        { "stmt": "app", "node": "Orders", "port": "events", "inner": { "node": "H", "port": "h" } }
    ]));
    assert_eq!(
        cascade(
            &mut ws,
            json!({ "stmt": "delete",
                    "edge": { "stmt": "conn-edge", "conn": "c",
                              "source": { "node": "A", "port": "out" },
                              "target": { "node": "Orders", "port": "events" } } })
        ),
        vec!["A.out c Orders.events"]
    );
    // The delegated port survives through the application — legal but suspect.
    let f = findings(&mut ws, json!({ "stmt": "check" }));
    assert!(
        f.iter().any(
            |f| matches!(f, Finding::DelegatedPortWithoutConnections { port }
            if port == "Orders.events")
        ),
        "expected delegated-port finding, got {f:?}"
    );
}

#[test]
fn deleting_a_node_named_by_a_route_takes_the_delegation() {
    let mut ws = routing_example();
    let c = cascade(&mut ws, json!({ "stmt": "delete", "node": "OrderCreated" }));
    assert!(c.contains(&"def node OrderCreated".to_string()));
    assert!(c.contains(&"Shipping.shipping_events send(OrderCreated) Orders.events".to_string()));
    assert!(c.contains(&"Orders.events(OrderCreated) = OrderHandler.handle".to_string()));
    // The unrelated qualified delegation stays.
    assert!(
        dump_pseudo(&ws)
            .contains(&"Orders.events(PaymentFailed) = PaymentHandler.handle".to_string())
    );
}

// ---- rename -----------------------------------------------------------------

#[test]
fn rename_is_reference_safe() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Orders" },
        { "stmt": "define", "conn": "confirm", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "target": { "anchor": "Service", "rel": "type_of" } },
        { "stmt": "conn-edge", "conn": "confirm",
          "source": { "node": "Payments", "port": "send" }, "target": { "node": "Orders", "port": "recv" } }
    ]));
    outcomes(
        &mut ws,
        json!([
            { "stmt": "rename", "node": "Payments", "to": "PaySvc" },
            { "stmt": "rename", "node": "Service", "to": "Kind" }
        ]),
    );
    let dump = dump_pseudo(&ws).join("\n");
    assert!(dump.contains("PaySvc.send confirm Orders.recv"));
    assert!(dump.contains("def conn confirm := (Kind type_of *) -> (Kind type_of *)"));
    assert!(!dump.contains("Payments"));
    assert!(!dump.contains("Service"));
}

// ---- patterns and transitivity ---------------------------------------------

#[test]
fn patterns_follow_the_transitive_closure() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "RestService" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "RestService" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "RestService", "target": "Payments" },
        { "stmt": "define", "node": "Api" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Api" },
        { "stmt": "define", "conn": "calls", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "target": { "anchor": "Service", "rel": "type_of" } }
    ]));
    // Payments is a Service only through RestService — a virtual pair.
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "calls",
                "source": { "node": "Api", "port": "out" }, "target": { "node": "Payments", "port": "recv" } })
    )));
}

#[test]
fn non_transitive_relations_match_single_steps_only() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "rel": "r", "directed": true, "source": "*", "target": "*" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "C" },
        { "stmt": "define", "node": "D" },
        { "stmt": "rel-edge", "rel": "r", "source": "A", "target": "B" },
        { "stmt": "rel-edge", "rel": "r", "source": "B", "target": "C" },
        { "stmt": "define", "rel": "needs", "directed": true,
          "source": { "anchor": "A", "rel": "r" }, "target": "*" }
    ]));
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "rel-edge", "rel": "needs", "source": "B", "target": "D" })
    )));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rel-edge", "rel": "needs", "source": "C", "target": "D" })
        ),
        ErrorCode::ShapeViolation
    );
}

// ---- subgraph queries -------------------------------------------------------

#[test]
fn query_scopes_open_chains_and_subtrees() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "A.B" },
        { "stmt": "define", "node": "A.B.C" },
        { "stmt": "define", "node": "D" },
        { "stmt": "define", "node": "D.X" }
    ]));
    // An empty list opens nothing: the top level only. Absent = everything.
    assert_eq!(
        node_ids(&mut ws, json!({ "stmt": "query", "scopes": [] })),
        vec!["A", "D"]
    );
    assert_eq!(
        node_ids(&mut ws, json!({ "stmt": "query" })),
        vec!["A", "A.B", "A.B.C", "D", "D.X"]
    );
    // An entry opens the chain from the root down to it plus its whole
    // subtree; unrelated scopes stay closed.
    assert_eq!(
        node_ids(&mut ws, json!({ "stmt": "query", "scopes": ["A.B"] })),
        vec!["A", "A.B", "A.B.C", "D"]
    );
}

#[test]
fn query_types_match_instances_transitively() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "RestService" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Billing" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "RestService" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "RestService", "target": "Payments" },
        { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" },
        { "stmt": "rel-edge", "rel": "dep", "source": "Payments", "target": "Billing" },
        { "stmt": "rel-edge", "rel": "dep", "source": "RestService", "target": "Payments" }
    ]));
    // Instances only, through the transitive closure; the type node itself
    // does not match.
    let (nodes, _) = graph(&mut ws, json!({ "stmt": "query", "types": ["Service"] }));
    let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, ["RestService", "Payments"]);
    // Node meta lists every classifier, transitively.
    assert_eq!(nodes[0].types, ["Service"]);
    assert_eq!(nodes[1].types, ["Service", "RestService"]);
    // Edges survive only with both attachments in the slice: the dep edge to
    // Billing and the classifier edge from Service are cut.
    assert_eq!(
        edge_values(&mut ws, json!({ "stmt": "query", "types": ["Service"] })),
        vec![
            json!({ "kind": "relation", "type": "type_of", "directed": true,
                    "source": "RestService", "target": "Payments" }),
            json!({ "kind": "relation", "type": "dep", "directed": true,
                    "source": "RestService", "target": "Payments" }),
        ]
    );
}

#[test]
fn query_kinds_filter_edges_and_compose_with_scopes() {
    let mut ws = routing_example();
    let (_, apps) = graph(
        &mut ws,
        json!({ "stmt": "query", "kinds": ["application"] }),
    );
    assert_eq!(apps.len(), 2);
    assert!(apps.iter().all(|e| e.kind == EdgeKind::Application));
    let (_, most) = graph(
        &mut ws,
        json!({ "stmt": "query", "kinds": ["connection", "relation"] }),
    );
    assert_eq!(most.len(), 5);
    // Composition: an application's inner node is hidden at the top level, so
    // no application survives a top-level-only scope filter.
    let (nodes, edges) = graph(
        &mut ws,
        json!({ "stmt": "query", "kinds": ["application"], "scopes": [] }),
    );
    assert!(edges.is_empty());
    assert!(nodes.iter().all(|n| !n.id.contains('.')));
}

#[test]
fn query_filters_are_validated() {
    let mut ws = ws_with(json!([{ "stmt": "define", "node": "A" }]));
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "types": ["Nope"] })),
        ErrorCode::UnknownName
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "scopes": ["Nope"] })),
        ErrorCode::UnknownName
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "views": ["nope"] })),
        ErrorCode::UnknownName
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "kinds": ["nope"] })),
        ErrorCode::Parse
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "node": "A" })),
        ErrorCode::Parse
    );
}

// ---- findings ------------------------------------------------------------------

#[test]
fn empty_views_and_uninstantiated_types_are_findings() {
    let mut ws = Workspace::new();
    assert_eq!(
        findings(&mut ws, json!({ "stmt": "check" })),
        vec![],
        "the stdlib is not reported"
    );
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "view": "lonely" },
            { "stmt": "define", "rel": "r", "directed": true, "source": "*", "target": "*" },
            { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" }
        ]),
    );
    let f = findings(&mut ws, json!({ "stmt": "check" }));
    assert!(f.contains(&Finding::EmptyView {
        view: "lonely".into()
    }));
    assert!(f.contains(&Finding::TypeWithoutInstances {
        type_kind: "rel",
        name: "r".into()
    }));
    assert!(f.contains(&Finding::TypeWithoutInstances {
        type_kind: "conn",
        name: "c".into()
    }));
}

// ---- serialization shapes ---------------------------------------------------

#[test]
fn outcome_and_finding_json_shapes_match_the_spec() {
    let mut ws = ws_with(json!([{ "stmt": "define", "node": "A" }]));
    let r = ws.handle(&json!({ "statements": [
        { "stmt": "define", "node": "A" },
        { "stmt": "delete", "node": "A" }
    ]}));
    let v: Value = serde_json::to_value(&r).unwrap();
    assert_eq!(v["status"], "ok");
    assert_eq!(v["results"][0], json!({ "result": "noop" }));
    assert_eq!(v["results"][1]["result"], "applied");
    assert_eq!(
        v["results"][1]["cascade"][0],
        json!({ "stmt": "define", "node": "A" })
    );

    // The graph result: nodes and edges, empty meta omitted.
    let r = ws.handle(&json!({ "statements": [
        { "stmt": "define", "node": "B" },
        { "stmt": "query" }
    ]}));
    let v: Value = serde_json::to_value(&r).unwrap();
    assert_eq!(v["results"][1]["result"], "graph");
    assert_eq!(
        v["results"][1]["nodes"],
        json!([{ "id": "B", "name": "B" }])
    );
    assert_eq!(v["results"][1]["edges"], json!([]));
}
