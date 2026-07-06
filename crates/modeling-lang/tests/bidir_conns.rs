//! Per-direction carriers on connection types: request/response (`->P, <-Q`),
//! pull (`->, <-Q`), lane checks, edge identity, cascades, drift findings.

mod common;

use common::*;
use modeling_lang::{ErrorCode, Finding, Workspace};
use serde_json::json;

/// UI/Auth request-response fixture: `login` carries LoginForm forward and
/// AuthResponse back.
fn login_example() -> Workspace {
    ws_with(json!([
        { "stmt": "define", "node": "LoginForm" },
        { "stmt": "define", "node": "AuthResponse" },
        { "stmt": "define", "node": "UI" },
        { "stmt": "define", "node": "AuthService" },
        { "stmt": "define", "conn": "login", "directed": true,
          "source": "*",
          "carrier": { "node": "LoginForm" },
          "rev_carrier": { "node": "AuthResponse" },
          "target": "*" },
        { "stmt": "conn-edge", "conn": "login",
          "source": { "node": "UI", "port": "login" },
          "carrier": "LoginForm",
          "rev_carrier": "AuthResponse",
          "target": { "node": "AuthService", "port": "handle_login" } }
    ]))
}

#[test]
fn bidir_conn_defines_and_instantiates() {
    let mut ws = login_example();
    // Identical restatements no-op — for the definition and the edge alike.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "conn": "login", "directed": true,
                "source": "*",
                "carrier": { "node": "LoginForm" },
                "rev_carrier": { "node": "AuthResponse" },
                "target": "*" })
    )));
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "login",
                "source": { "node": "UI", "port": "login" },
                "carrier": "LoginForm",
                "rev_carrier": "AuthResponse",
                "target": { "node": "AuthService", "port": "handle_login" } })
    )));
    // A divergent rev lane is a redeclaration, not a silent change.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "conn": "login", "directed": true,
                    "source": "*",
                    "carrier": { "node": "LoginForm" },
                    "target": "*" })
        ),
        ErrorCode::Redeclared
    );
    // Query results expose both lanes.
    let edges = edge_values(&mut ws, json!({ "stmt": "query", "kinds": ["connection"] }));
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["carrier"], "LoginForm");
    assert_eq!(edges[0]["rev_carrier"], "AuthResponse");
    assert_eq!(edges[0]["source_port"], "login");
    assert_eq!(edges[0]["target_port"], "handle_login");
}

#[test]
fn rev_only_lane_models_pull() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Payload" },
        { "stmt": "define", "node": "Client" },
        { "stmt": "define", "node": "Store" },
        { "stmt": "define", "conn": "fetch", "directed": true,
          "source": "*", "rev_carrier": { "node": "Payload" }, "target": "*" },
        { "stmt": "conn-edge", "conn": "fetch",
          "source": { "node": "Client", "port": "get" },
          "rev_carrier": "Payload",
          "target": { "node": "Store", "port": "serve" } }
    ]));
    let edges = edge_values(&mut ws, json!({ "stmt": "query", "kinds": ["connection"] }));
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].get("carrier"), None);
    assert_eq!(edges[0]["rev_carrier"], "Payload");
}

#[test]
fn undirected_types_reject_a_rev_lane() {
    let mut ws = ws_with(json!([{ "stmt": "define", "node": "Chunk" }]));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "conn": "pipe", "directed": false,
                    "source": "*", "rev_carrier": { "node": "Chunk" }, "target": "*" })
        ),
        ErrorCode::Parse
    );
}

#[test]
fn lane_arity_is_checked_per_lane() {
    let mut ws = login_example();
    // Omitting the rev carrier of a lane that has one: required.
    let e = err(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "login",
                 "source": { "node": "UI", "port": "login" },
                 "carrier": "LoginForm",
                 "target": { "node": "AuthService", "port": "handle_login" } }]),
    )
    .1;
    assert_eq!(e.code, ErrorCode::CarrierRequired);
    assert!(
        e.message.contains("reverse"),
        "names the lane: {}",
        e.message
    );
    // Naming a rev carrier on a type without a reverse lane: forbidden.
    outcomes(
        &mut ws,
        json!([{ "stmt": "define", "conn": "notify", "directed": true,
                 "source": "*", "carrier": { "node": "LoginForm" }, "target": "*" }]),
    );
    let e = err(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "notify",
                 "source": { "node": "UI", "port": "out" },
                 "carrier": "LoginForm",
                 "rev_carrier": "AuthResponse",
                 "target": { "node": "AuthService", "port": "in" } }]),
    )
    .1;
    assert_eq!(e.code, ErrorCode::CarrierForbidden);
    assert!(
        e.message.contains("reverse"),
        "names the lane: {}",
        e.message
    );
    // A rev carrier that fails the lane's pattern: shape violation.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "login",
                    "source": { "node": "UI", "port": "login" },
                    "carrier": "LoginForm",
                    "rev_carrier": "LoginForm",
                    "target": { "node": "AuthService", "port": "handle_login" } })
        ),
        ErrorCode::ShapeViolation
    );
}

#[test]
fn rev_carrier_is_part_of_edge_identity() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Query" },
        { "stmt": "define", "node": "ResultA" },
        { "stmt": "define", "node": "ResultB" },
        { "stmt": "define", "node": "Client" },
        { "stmt": "define", "node": "Server" },
        { "stmt": "define", "conn": "rpc", "directed": true,
          "source": "*", "carrier": { "node": "Query" }, "rev_carrier": "*", "target": "*" }
    ]));
    let edge = |rc: &str| {
        json!({ "stmt": "conn-edge", "conn": "rpc",
                "source": { "node": "Client", "port": "call" },
                "carrier": "Query", "rev_carrier": rc,
                "target": { "node": "Server", "port": "serve" } })
    };
    assert!(is_applied(&outcome(&mut ws, edge("ResultA"))));
    // A different reverse payload is a different edge, not a restatement.
    assert!(is_applied(&outcome(&mut ws, edge("ResultB"))));
    assert!(is_noop(&outcome(&mut ws, edge("ResultA"))));
    let edges = edge_values(&mut ws, json!({ "stmt": "query", "kinds": ["connection"] }));
    assert_eq!(edges.len(), 2);
    // Deletion addresses one edge structurally, rev carrier included.
    let removed = cascade(
        &mut ws,
        json!({ "stmt": "delete", "edge": {
            "stmt": "conn-edge", "conn": "rpc",
            "source": { "node": "Client", "port": "call" },
            "carrier": "Query", "rev_carrier": "ResultA",
            "target": { "node": "Server", "port": "serve" } } }),
    );
    assert_eq!(removed.len(), 1);
    let edges = edge_values(&mut ws, json!({ "stmt": "query", "kinds": ["connection"] }));
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["rev_carrier"], "ResultB");
}

#[test]
fn deleting_a_rev_carried_node_cascades_over_the_edge() {
    let mut ws = login_example();
    let removed = cascade(&mut ws, json!({ "stmt": "delete", "node": "AuthResponse" }));
    // The node, the conn type whose rev lane pattern names it, and the edge.
    assert!(removed.iter().any(|s| s.contains("def node AuthResponse")));
    assert!(removed.iter().any(|s| s.contains("conn login")));
    assert!(removed.iter().any(|s| s.contains("UI.login")));
    let edges = edge_values(&mut ws, json!({ "stmt": "query", "kinds": ["connection"] }));
    assert!(edges.is_empty());
}

#[test]
fn rev_lane_drift_is_a_finding() {
    let mut ws = login_example();
    // Loosen the lane so the edge stays, then point it somewhere the edge's
    // rev carrier no longer matches.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Denial" },
            { "stmt": "redefine", "conn": "login", "directed": true,
              "source": "*",
              "carrier": { "node": "LoginForm" },
              "rev_carrier": { "node": "Denial" },
              "target": "*" }
        ]),
    );
    let fs = findings(&mut ws, json!({ "stmt": "check" }));
    assert!(
        fs.iter().any(|f| matches!(
            f,
            Finding::ShapeDrift { slot, actual, .. }
                if slot == "rev_carrier" && actual == "AuthResponse"
        )),
        "expected rev_carrier drift, got {fs:?}"
    );
}

#[test]
fn dump_with_rev_carriers_replays_idempotently() {
    let ws = login_example();
    let dump = ws.model().dump();
    let mut ws2 = Workspace::new();
    let results = ws2.execute(&dump).expect("dump replays");
    assert!(results.iter().all(is_applied));
    let results = ws2.execute(&dump).expect("dump replays over itself");
    assert!(results.iter().all(is_noop));
    assert_eq!(dump_pseudo(&ws), dump_pseudo(&ws2));
}
