//! Declared ports: `define node` with a `ports` list. Declared ports exist
//! before any edge, survive losing their last edge, fix type/side at first
//! use, and are replaced as a set by `redefine`.

mod common;

use common::*;
use modeling_lang::{ErrorCode, Finding, Outcome, Workspace};
use serde_json::json;

fn auth_example() -> Workspace {
    ws_with(json!([
        { "stmt": "define", "node": "AuthService",
          "ports": ["handle_login", "handle_get_token", "send_audit_log"] },
        { "stmt": "define", "node": "UI", "ports": ["login"] },
        { "stmt": "define", "conn": "login", "directed": true, "source": "*", "target": "*" }
    ]))
}

#[test]
fn declared_ports_exist_before_any_edge() {
    let mut ws = auth_example();
    // The definition is part of the node's statement: dumps restate it.
    assert!(dump_pseudo(&ws).contains(
        &"def node AuthService:\n  port handle_get_token\n  port handle_login\n  port send_audit_log".to_string()
    ));
    // Unattached declared ports are findings, not errors.
    let fs = findings(&mut ws, json!({ "stmt": "check" }));
    let unused: Vec<_> = fs
        .iter()
        .filter(|f| matches!(f, Finding::UnusedPort { .. }))
        .collect();
    assert_eq!(unused.len(), 4);
}

#[test]
fn port_claims_compare_as_sets() {
    let mut ws = auth_example();
    // Same set, different order: identical restatement.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "AuthService",
                "ports": ["send_audit_log", "handle_login", "handle_get_token"] })
    )));
    // No `ports` field: no claim about the port set.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "AuthService" })
    )));
    // A divergent claim is a redeclaration.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "node": "AuthService", "ports": ["handle_login"] })
        ),
        ErrorCode::Redeclared
    );
    // Malformed port lists are parse errors.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "node": "X", "ports": ["a", "a"] })
        ),
        ErrorCode::Parse
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "node": "X", "ports": ["not a port"] })
        ),
        ErrorCode::Parse
    );
}

#[test]
fn first_use_fixes_type_and_side_of_a_declared_port() {
    let mut ws = auth_example();
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "login",
                "source": { "node": "UI", "port": "login" },
                "target": { "node": "AuthService", "port": "handle_login" } })
    )));
    // The declared port is now fixed: a disagreeing use is rejected.
    outcomes(
        &mut ws,
        json!([{ "stmt": "define", "conn": "audit", "directed": true, "source": "*", "target": "*" }]),
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "audit",
                    "source": { "node": "UI", "port": "login" },
                    "target": { "node": "AuthService", "port": "send_audit_log" } })
        ),
        ErrorCode::PortTypeConflict
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "login",
                    "source": { "node": "AuthService", "port": "handle_login" },
                    "target": { "node": "UI", "port": "login" } })
        ),
        ErrorCode::PortSideConflict
    );
}

#[test]
fn delegating_a_declared_unattached_port_is_rejected() {
    let mut ws = auth_example();
    outcomes(
        &mut ws,
        json!([{ "stmt": "define", "node": "AuthService.LoginHandler", "ports": ["handle"] }]),
    );
    let e = err(
        &mut ws,
        json!([{ "stmt": "app", "node": "AuthService", "port": "handle_login",
                 "inner": { "node": "LoginHandler", "port": "handle" } }]),
    )
    .1;
    assert_eq!(e.code, ErrorCode::NoOuterPort);
    assert!(
        e.message.contains("declared"),
        "distinguishes the declared case: {}",
        e.message
    );
    // Once a connection attaches, delegation works — into a declared inner port.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "conn-edge", "conn": "login",
              "source": { "node": "UI", "port": "login" },
              "target": { "node": "AuthService", "port": "handle_login" } },
            { "stmt": "app", "node": "AuthService", "port": "handle_login",
              "inner": { "node": "LoginHandler", "port": "handle" } }
        ]),
    );
}

#[test]
fn declared_ports_survive_their_edges() {
    let mut ws = auth_example();
    outcomes(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "login",
                 "source": { "node": "UI", "port": "login" },
                 "target": { "node": "AuthService", "port": "handle_login" } }]),
    );
    cascade(
        &mut ws,
        json!({ "stmt": "delete", "edge": {
            "stmt": "conn-edge", "conn": "login",
            "source": { "node": "UI", "port": "login" },
            "target": { "node": "AuthService", "port": "handle_login" } } }),
    );
    // Both ends were declared: both survive, still part of their definitions.
    assert!(dump_pseudo(&ws).contains(&"def node UI:\n  port login".to_string()));
    // A use-created port would have been swept (existing behavior, unchanged).
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Probe" },
            { "stmt": "conn-edge", "conn": "login",
              "source": { "node": "Probe", "port": "out" },
              "target": { "node": "AuthService", "port": "handle_login" } }
        ]),
    );
    cascade(
        &mut ws,
        json!({ "stmt": "delete", "edge": {
            "stmt": "conn-edge", "conn": "login",
            "source": { "node": "Probe", "port": "out" },
            "target": { "node": "AuthService", "port": "handle_login" } } }),
    );
    assert!(dump_pseudo(&ws).contains(&"def node Probe".to_string()));
}

#[test]
fn deleting_the_fixed_type_releases_a_declared_port() {
    let mut ws = auth_example();
    outcomes(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "login",
                 "source": { "node": "UI", "port": "login" },
                 "target": { "node": "AuthService", "port": "handle_login" } }]),
    );
    cascade(&mut ws, json!({ "stmt": "delete", "conn": "login" }));
    // The binding is released with the type: a fresh first use may fix a new
    // type — and the opposite side.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "conn": "audit", "directed": true, "source": "*", "target": "*" },
            { "stmt": "conn-edge", "conn": "audit",
              "source": { "node": "AuthService", "port": "handle_login" },
              "target": { "node": "UI", "port": "login" } }
        ]),
    );
}

#[test]
fn redefine_replaces_the_declared_set() {
    let mut ws = auth_example();
    outcomes(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "login",
                 "source": { "node": "UI", "port": "login" },
                 "target": { "node": "AuthService", "port": "handle_login" } }]),
    );
    // Drop two declared ports (one attached, one not), add a fresh one.
    let o = outcome(
        &mut ws,
        json!({ "stmt": "redefine", "node": "AuthService",
                "ports": ["handle_get_token", "revoke_token"] }),
    );
    assert!(is_applied(&o));
    assert!(dump_pseudo(&ws).contains(
        &"def node AuthService:\n  port handle_get_token\n  port revoke_token".to_string()
    ));
    // The attached ex-declared port demoted to use-created: the edge still
    // stands; deleting it now sweeps the port.
    let edges = edge_values(&mut ws, json!({ "stmt": "query", "kinds": ["connection"] }));
    assert_eq!(edges.len(), 1);
    cascade(
        &mut ws,
        json!({ "stmt": "delete", "edge": {
            "stmt": "conn-edge", "conn": "login",
            "source": { "node": "UI", "port": "login" },
            "target": { "node": "AuthService", "port": "handle_login" } } }),
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "app", "node": "AuthService", "port": "handle_login",
                    "inner": { "node": "X", "port": "y" } })
        ),
        ErrorCode::NoOuterPort
    );
    // Identical redefinition (set order aside) is a no-op.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "redefine", "node": "AuthService",
                "ports": ["revoke_token", "handle_get_token"] })
    )));
    // A `redefine` without `ports` leaves the declared set alone.
    match outcome(
        &mut ws,
        json!({ "stmt": "redefine", "node": "AuthService" }),
    ) {
        Outcome::Noop => {}
        o => panic!("empty scope, unchanged ports: expected noop, got {o:?}"),
    }
}

#[test]
fn dump_with_declared_ports_replays_idempotently() {
    let mut ws = auth_example();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "AuthService.LoginHandler", "ports": ["handle"] },
            { "stmt": "conn-edge", "conn": "login",
              "source": { "node": "UI", "port": "login" },
              "target": { "node": "AuthService", "port": "handle_login" } },
            { "stmt": "app", "node": "AuthService", "port": "handle_login",
              "inner": { "node": "LoginHandler", "port": "handle" } }
        ]),
    );
    let dump = ws.model().dump();
    let mut ws2 = Workspace::new();
    let results = ws2.execute(&dump).expect("dump replays");
    assert!(results.iter().all(is_applied));
    let results = ws2.execute(&dump).expect("dump replays over itself");
    assert!(results.iter().all(is_noop));
    assert_eq!(dump_pseudo(&ws), dump_pseudo(&ws2));
}
