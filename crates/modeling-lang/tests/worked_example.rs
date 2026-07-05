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
fn ports_query_matches_spec_output() {
    let mut ws = worked_example();
    assert_eq!(
        pseudo(&mut ws, json!({ "stmt": "ports", "node": "Orders" })),
        vec![
            "Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);",
            "Orders.handle_confirmation = ConfirmationHandler(handle_confirmation);",
        ]
    );
}

#[test]
fn ports_of_inner_node_include_the_application() {
    let mut ws = worked_example();
    assert_eq!(
        pseudo(
            &mut ws,
            json!({ "stmt": "ports", "node": "Orders.ConfirmationHandler" })
        ),
        vec!["Orders.handle_confirmation = ConfirmationHandler(handle_confirmation);"]
    );
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
    let dumped = statements(&mut ws, json!({ "stmt": "dump" }));
    let lines: Vec<String> = dumped.iter().map(Statement::pseudo).collect();
    assert!(lines.contains(&"def node Orders;".to_string()));
    assert!(lines.contains(&"def node Orders.ConfirmationHandler;".to_string()));

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
    let mut ws = worked_example();
    let dumped = statements(&mut ws, json!({ "stmt": "dump" }));
    for stmt in &dumped {
        let v = stmt.to_value();
        let back = modeling_lang::parse_statement(&v).expect("rendered statements parse");
        assert_eq!(&back, stmt);
    }
}
