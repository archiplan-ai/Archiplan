//! The flow-query filters (`archi/requirements/modeling-language/queries-compose-filters.md`):
//! `carriers` slices a datum's flow — connection edges carrying the named
//! node, directly or via a classifying type, plus the nodes related to
//! them; `edge_types` slices by rel/conn type name, and never passes an
//! application, which is untyped.

mod common;

use common::*;
use modeling_lang::{ErrorCode, Workspace};
use serde_json::json;

/// Payments —OrderId→ Orders —CredToken→ Ledger —Receipt/⇐CredToken→ Audit,
/// with `Secret` classifying `CredToken`, and a delegation into Orders.
fn flow_model() -> Workspace {
    ws_with(json!([
        { "stmt": "define", "node": "Secret" },
        { "stmt": "define", "node": "CredToken" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Secret", "target": "CredToken" },
        { "stmt": "define", "node": "OrderId" },
        { "stmt": "define", "node": "Receipt" },

        { "stmt": "define", "conn": "wire", "directed": true,
          "source": "*", "carrier": "*", "target": "*" },
        { "stmt": "define", "conn": "bus", "directed": true,
          "source": "*", "carrier": "*", "rev_carrier": "*", "target": "*" },

        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.Handler" },
        { "stmt": "define", "node": "Ledger" },
        { "stmt": "define", "node": "Audit" },

        { "stmt": "conn-edge", "conn": "wire",
          "source": { "node": "Payments", "port": "orders_out" },
          "carrier": "OrderId",
          "target": { "node": "Orders", "port": "inn" } },
        { "stmt": "app", "node": "Orders", "port": "inn",
          "inner": { "node": "Handler", "port": "inn" } },
        { "stmt": "conn-edge", "conn": "wire",
          "source": { "node": "Orders", "port": "cred_out" },
          "carrier": "CredToken",
          "target": { "node": "Ledger", "port": "inn" } },
        { "stmt": "conn-edge", "conn": "bus",
          "source": { "node": "Ledger", "port": "audit_out" },
          "carrier": "Receipt", "rev_carrier": "CredToken",
          "target": { "node": "Audit", "port": "inn" } },
    ]))
}

#[test]
fn carriers_slice_the_flow_of_a_datum() {
    let mut ws = flow_model();

    // The flow of CredToken: the wire that carries it forward and the bus
    // that carries it back — with only the related nodes admitted:
    // attachments plus carriers (Receipt rides in on the bus edge).
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query", "carriers": ["CredToken"] }));
    let types: Vec<_> = edges.iter().map(|e| e.type_name.as_deref().unwrap()).collect();
    assert_eq!(types, ["wire", "bus"]);
    assert_eq!(
        nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
        ["CredToken", "Receipt", "Orders", "Ledger", "Audit"]
    );

    // Naming the classifying type means its instances, as in the node
    // `types` filter; naming another datum slices its own flow.
    let (_, via_type) = graph(&mut ws, json!({ "stmt": "query", "carriers": ["Secret"] }));
    assert_eq!(via_type.len(), 2);
    let (_, orders) = graph(&mut ws, json!({ "stmt": "query", "carriers": ["OrderId"] }));
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].source, "Payments");

    // The empty list is the most restrictive filter; relations and the
    // application carry nothing and never pass a carriers filter.
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query", "carriers": [] }));
    assert!(nodes.is_empty() && edges.is_empty());
}

#[test]
fn edge_types_slice_by_name_and_never_pass_applications() {
    let mut ws = flow_model();

    // By conn name: both wires, not the bus, not the type_of relation, not
    // the application. Nodes are unrestricted — edge_types is not an
    // edge-driven admitter the way views and carriers are.
    let (nodes, edges) = graph(&mut ws, json!({ "stmt": "query", "edge_types": ["wire"] }));
    assert_eq!(edges.len(), 2);
    assert!(edges.iter().all(|e| e.type_name.as_deref() == Some("wire")));
    assert_eq!(nodes.len(), 9, "every node of the model");

    // By rel name; names of both kinds compose into one filter.
    let (_, rels) = graph(&mut ws, json!({ "stmt": "query", "edge_types": ["type_of"] }));
    assert_eq!(rels.len(), 1);
    assert_eq!((rels[0].source.as_str(), rels[0].target.as_str()), ("Secret", "CredToken"));
    let (_, both) = graph(
        &mut ws,
        json!({ "stmt": "query", "edge_types": ["wire", "type_of"] }),
    );
    assert_eq!(both.len(), 3);

    // The empty list matches nothing; composition with carriers is AND and
    // the related-node admission follows the surviving edges.
    let (_, none) = graph(&mut ws, json!({ "stmt": "query", "edge_types": [] }));
    assert!(none.is_empty());
    let (nodes, edges) = graph(
        &mut ws,
        json!({ "stmt": "query", "carriers": ["CredToken"], "edge_types": ["bus"] }),
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(
        nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
        ["CredToken", "Receipt", "Ledger", "Audit"]
    );
}

#[test]
fn filter_names_resolve_or_error() {
    let mut ws = flow_model();
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "carriers": ["Nope"] })),
        ErrorCode::UnknownName
    );
    let (_, e) = err(
        &mut ws,
        json!([{ "stmt": "query", "edge_types": ["nope"] }]),
    );
    assert_eq!(e.code, ErrorCode::UnknownName);
    assert!(e.message.contains("edge-type"), "{}", e.message);
    // Strict keys: a misspelled filter is a parse error, not a silent no-op.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "query", "carrier": ["CredToken"] })),
        ErrorCode::Parse
    );
}
