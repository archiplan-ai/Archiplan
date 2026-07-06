//! The worked example from `requirements/modeling-lang/modeling-lang.md`,
//! end to end: build, query, classify layers, round-trip through `dump`.

mod common;

use common::*;
use modeling_lang::{Layer, Statement, Workspace};
use serde_json::{Value, json};

fn worked_example_batch() -> Value {
    json!([
        { "stmt": "define", "rel": "of_sort", "trans": true, "directed": true,
          "source": "*", "target": "*" },

        { "stmt": "define", "node": "Functional" },
        { "stmt": "define", "node": "Data" },

        { "stmt": "define", "node": "Service" },
        { "stmt": "rel-edge", "rel": "of_sort", "source": "Service", "target": "Functional" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Orders" },

        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Orders" },

        { "stmt": "define", "node": "OrderId" },
        { "stmt": "rel-edge", "rel": "of_sort", "source": "OrderId", "target": "Data" },

        { "stmt": "define", "conn": "confirm", "directed": true,
          "source":  { "anchor": "Service", "rel": "type_of" },
          "carrier": { "node": "OrderId" },
          "target":  { "anchor": "Service", "rel": "type_of" } },
        { "stmt": "conn-edge", "conn": "confirm",
          "source":  { "node": "Payments", "port": "send_confirmation" },
          "carrier": "OrderId",
          "target":  { "node": "Orders", "port": "handle_confirmation" } },

        { "stmt": "define", "node": "Orders.ConfirmationHandler" },
        { "stmt": "app", "node": "Orders", "port": "handle_confirmation",
          "inner": { "node": "ConfirmationHandler", "port": "handle_confirmation" } }
    ])
}

pub fn worked_example() -> Workspace {
    ws_with(worked_example_batch())
}

#[test]
fn worked_example_applies_and_bumps_revision_once() {
    let mut ws = Workspace::new();
    let results = ws
        .execute_values(worked_example_batch().as_array().unwrap())
        .expect("worked example applies");
    assert!(results.iter().all(is_applied));
    assert_eq!(
        ws.revision(),
        1,
        "one model-changing batch = one revision bump"
    );
}

#[test]
fn query_returns_the_slice_with_meta() {
    let mut ws = worked_example();
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query" }));
    assert_eq!(
        nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
        [
            "Functional",
            "Data",
            "Service",
            "Payments",
            "Orders",
            "OrderId",
            "Orders.ConfirmationHandler"
        ]
    );
    // Nodes carry their classifiers and the ports the result's edges use.
    let orders = serde_json::to_value(nodes.iter().find(|n| n.id == "Orders").unwrap()).unwrap();
    assert_eq!(
        orders,
        json!({ "id": "Orders", "name": "Orders", "types": ["Service"],
                "ports": [{ "name": "handle_confirmation", "conn": "confirm", "side": "target" }] })
    );
    // Edges carry kind, type and attachment meta; empty fields are omitted.
    let vals: Vec<serde_json::Value> = edges
        .iter()
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();
    assert!(vals.contains(
        &json!({ "kind": "connection", "type": "confirm", "directed": true,
            "source": "Payments", "source_port": "send_confirmation",
            "target": "Orders", "target_port": "handle_confirmation",
            "carrier": "OrderId" })
    ));
    assert!(vals.contains(&json!({ "kind": "application",
            "source": "Orders", "source_port": "handle_confirmation",
            "target": "Orders.ConfirmationHandler", "target_port": "handle_confirmation" })));
    assert_eq!(edges.len(), 6, "4 relations, 1 connection, 1 application");
}

#[test]
fn query_filters_compose_on_the_worked_example() {
    let mut ws = worked_example();
    // Top level only: the application into Orders' scope is folded away.
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query", "scopes": [] }));
    assert_eq!(nodes.len(), 6);
    assert_eq!(edges.len(), 5);
    assert!(!nodes.iter().any(|n| n.id.contains('.')));
    // Only Service instances: classifier edges to the (excluded) type node
    // are cut; the connection between the instances survives.
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query", "types": ["Service"] }));
    assert_eq!(
        nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
        ["Payments", "Orders"]
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].type_name.as_deref(), Some("confirm"));
}

#[test]
fn layers_follow_type_of() {
    let ws = worked_example();
    let m = ws.model();
    // Types appear on the left of `type_of`; everything else is a term —
    // including Functional, which classifies only through `of_sort`.
    assert_eq!(m.layer_of("Service"), Some(Layer::Epistemic));
    assert_eq!(m.layer_of("Payments"), Some(Layer::Epistatic));
    assert_eq!(m.layer_of("OrderId"), Some(Layer::Epistatic));
    assert_eq!(m.layer_of("Functional"), Some(Layer::Epistatic));
    assert_eq!(
        m.layer_of("Orders.ConfirmationHandler"),
        Some(Layer::Epistatic)
    );
    assert_eq!(m.layer_of("Nope"), None);
}

#[test]
fn dump_round_trips_idempotently() {
    let mut ws = worked_example();
    let dumped = ws.model().dump();
    let lines: Vec<String> = dumped.iter().map(Statement::pseudo).collect();
    assert!(lines.contains(&"def node Orders".to_string()));
    assert!(lines.contains(&"def node Orders.ConfirmationHandler".to_string()));

    let mut replayed = Workspace::new();
    replayed
        .execute(&dumped)
        .expect("a dump replays into a fresh workspace");
    assert_eq!(
        replayed.model().dump(),
        dumped,
        "replayed model dumps identically"
    );

    // Replaying a dump over the model it came from is all noops.
    let outcomes = ws.execute(&dumped).expect("replay over self");
    assert!(outcomes.iter().all(is_noop));
}

#[test]
fn check_is_clean() {
    let mut ws = worked_example();
    assert_eq!(findings(&mut ws, json!({ "stmt": "check" })), vec![]);
}

#[test]
fn statement_objects_round_trip_through_serde() {
    let ws = worked_example();
    let dumped = ws.model().dump();
    for stmt in &dumped {
        let v = stmt.to_value();
        let back = modeling_lang::parse_statement(&v).expect("rendered statements parse");
        assert_eq!(&back, stmt);
    }
}
