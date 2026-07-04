//! The worked example from `requirements/modeling-lang/modeling-lang.md`,
//! end to end: build, query, classify layers, round-trip through `dump`.

mod common;

use common::*;
use modeling_lang::{Layer, Outcome, Session};

const WORKED_EXAMPLE: &str = r#"
rel trans of_sort := * -> *;
# type_of comes from the stdlib

node Functional;
node Data;

node Service;
Service of_sort Functional;
node Payments;
node Orders;

# Payments and Orders are concrete services (terms of type Service)
Service type_of Payments;
Service type_of Orders;

node OrderId;
OrderId of_sort Data;

# a connection between two services' ports, carrying an OrderId
conn confirm := (Service type_of *) (OrderId)-> (Service type_of *);
Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);

# inside Orders, the boundary port is delegated to an inner handler
node Orders {
  node ConfirmationHandler;
  handle_confirmation = ConfirmationHandler(handle_confirmation);
}
"#;

pub fn worked_example() -> Session {
    session_with(WORKED_EXAMPLE)
}

#[test]
fn worked_example_applies() {
    let mut s = Session::new();
    let results = s.execute(WORKED_EXAMPLE).expect("worked example applies");
    // The reopening `node Orders { ... }` reports the node part as a noop and
    // the inner statements as applied.
    let Outcome::Block(inner) = &results.last().unwrap().outcome else {
        panic!("expected a block outcome");
    };
    assert!(
        is_noop(&inner[0].outcome),
        "restated `node Orders` is a noop"
    );
    assert!(inner[1..].iter().all(|r| is_applied(&r.outcome)));
}

#[test]
fn ports_query_matches_spec_output() {
    let mut s = worked_example();
    assert_eq!(
        statements(&mut s, "ports Orders"),
        vec![
            "Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);",
            "Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }",
        ]
    );
}

#[test]
fn ports_of_inner_node_include_the_application() {
    let mut s = worked_example();
    assert_eq!(
        statements(&mut s, "ports Orders.ConfirmationHandler"),
        vec!["Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }"]
    );
}

#[test]
fn layers_follow_type_of() {
    let s = worked_example();
    let m = s.model();
    // Types appear on the left of `type_of`; everything else is a term —
    // including Functional, which classifies only through `of_sort`.
    assert_eq!(m.layer_of("Service"), Some(Layer::Epistemic));
    assert_eq!(m.layer_of("Payments"), Some(Layer::Epistatic));
    assert_eq!(m.layer_of("Orders"), Some(Layer::Epistatic));
    assert_eq!(m.layer_of("OrderId"), Some(Layer::Epistatic));
    assert_eq!(m.layer_of("Functional"), Some(Layer::Epistatic));
    assert_eq!(
        m.layer_of("Orders.ConfirmationHandler"),
        Some(Layer::Epistatic)
    );
    assert_eq!(m.layer_of("Nope"), None);
}

#[test]
fn dump_round_trips() {
    let mut s = worked_example();
    let dumped = statements(&mut s, "dump");
    assert!(dumped.contains(&"node Orders;".to_string()));
    assert!(dumped.contains(&"Orders { node ConfirmationHandler; }".to_string()));

    let mut replayed = Session::new();
    replayed
        .execute(&dumped.join("\n"))
        .expect("a dump replays into a fresh session");
    assert_eq!(
        replayed.model().dump(),
        dumped,
        "replayed model dumps identically"
    );
}

#[test]
fn check_is_clean() {
    let mut s = worked_example();
    // The worked example is complete: every declared type has instances, the
    // delegated port has traffic, nothing drifted.
    assert_eq!(findings(&mut s, "check"), vec![]);
}
