//! One test per code in the error catalog, the definition preconditions, the
//! atomicity guarantees, and the request/response envelope.

mod common;

use common::*;
use modeling_lang::{ErrorCode, Workspace};
use serde_json::json;

#[test]
fn e_parse_schema_violations() {
    let mut ws = Workspace::new();
    // Unknown statement kind.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "nope" })),
        ErrorCode::Parse
    );
    // Unknown field for the subject.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "node": "A", "scope": "B" })
        ),
        ErrorCode::Parse
    );
    // A definition names exactly one subject.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "define" })),
        ErrorCode::Parse
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "node": "A", "view": "v" })
        ),
        ErrorCode::Parse
    );
    // Missing required field: `directed` has no default.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "rel": "r", "source": "*", "target": "*" })
        ),
        ErrorCode::Parse
    );
    // Ill-typed field.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "define", "node": 7 })),
        ErrorCode::Parse
    );
    // Malformed path.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "define", "node": "A..B" })),
        ErrorCode::Parse
    );
    // A pattern is "*", {node} or {anchor, rel}.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "rel": "r", "directed": true,
                    "source": "anything", "target": "*" })
        ),
        ErrorCode::Parse
    );
    // `redefine` does not apply to views.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "redefine", "view": "v" })),
        ErrorCode::Parse
    );
    // `delete` takes exactly one target.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "delete", "node": "A", "rel": "r" })
        ),
        ErrorCode::Parse
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "delete" })),
        ErrorCode::Parse
    );
    // `edge` must restate an edge statement.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "delete", "edge": { "stmt": "define", "node": "A" } })
        ),
        ErrorCode::Parse
    );
    // The subject of a parse error is the statement as submitted.
    let (_, e) = err(&mut ws, json!([{ "stmt": "nope" }]));
    assert_eq!(e.subject, Some(json!({ "stmt": "nope" })));
}

#[test]
fn e_parse_dotted_inner_end_of_application() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.Sub" },
        { "stmt": "define", "node": "Orders.Sub.Deep" },
        { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "c",
          "source": { "node": "A", "port": "out" }, "target": { "node": "Orders", "port": "p" } }
    ]));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "app", "node": "Orders", "port": "p",
                    "inner": { "node": "Sub.Deep", "port": "q" } })
        ),
        ErrorCode::Parse
    );
}

#[test]
fn e_unknown_name() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" }
    ]));
    // Unresolved reference in an edge.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rel-edge", "rel": "type_of", "source": "Ghost", "target": "A" })
        ),
        ErrorCode::UnknownName
    );
    // Unknown relation type.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rel-edge", "rel": "nope", "source": "A", "target": "B" })
        ),
        ErrorCode::UnknownName
    );
    // Undeclared view.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rel-edge", "rel": "type_of", "source": "A", "target": "B", "views": ["nope"] })
        ),
        ErrorCode::UnknownName
    );
    // A creation's container must exist: augmentation presupposes it.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "define", "node": "Ghost.Child" })),
        ErrorCode::UnknownName
    );
    // `redefine` of an element that does not exist.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "redefine", "node": "Ghost" })),
        ErrorCode::UnknownName
    );
    // Deleting a missing node / restating a missing edge.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "delete", "node": "Ghost" })),
        ErrorCode::UnknownName
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "delete", "edge": { "stmt": "rel-edge", "rel": "type_of", "source": "A", "target": "B" } })
        ),
        ErrorCode::UnknownName
    );
}

#[test]
fn e_dup_name_on_rename() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" }
    ]));
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "rename", "node": "A", "to": "B" })),
        ErrorCode::DupName
    );
    // Renaming to its own name is a no-op, not a collision.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "rename", "node": "A", "to": "A" })
    )));
}

#[test]
fn e_redeclared_on_divergent_define() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "view": "v" },
        { "stmt": "define", "rel": "r", "directed": true, "source": "*", "target": "*" }
    ]));
    // A node or view define never diverges: there is no body, restating is a no-op.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "A" })
    )));
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "view": "v" })
    )));
    // An identical rel define is a no-op; a divergent one is rejected.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "rel": "r", "directed": true, "source": "*", "target": "*" })
    )));
    let (_, e) = err(
        &mut ws,
        json!([{ "stmt": "define", "rel": "r", "directed": false, "source": "*", "target": "*" }]),
    );
    assert_eq!(e.code, ErrorCode::Redeclared);
    assert!(e.actual.is_some(), "the existing definition is included");
    assert!(e.hint.is_some(), "the matching redefine is suggested");
    // One namespace for edge types: a conn define under a rel's name diverges
    // by kind — for `redefine` too, which never crosses kinds.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "define", "conn": "r", "directed": true, "source": "*", "target": "*" })
        ),
        ErrorCode::Redeclared
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "redefine", "conn": "r", "directed": true, "source": "*", "target": "*" })
        ),
        ErrorCode::Redeclared
    );
}

#[test]
fn e_shape_violation_on_ends_and_carrier() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Service" },
        { "stmt": "define", "node": "Data" },
        { "stmt": "define", "node": "Payments" },
        { "stmt": "define", "node": "Invoice" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Data", "target": "Invoice" },
        { "stmt": "define", "conn": "calls", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "target": { "anchor": "Service", "rel": "type_of" } },
        { "stmt": "define", "conn": "send", "directed": true,
          "source": { "anchor": "Service", "rel": "type_of" },
          "carrier": { "anchor": "Data", "rel": "type_of" },
          "target": { "anchor": "Service", "rel": "type_of" } }
    ]));
    let (_, e) = err(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "calls",
                 "source": { "node": "Payments", "port": "out" },
                 "target": { "node": "Invoice", "port": "recv" } }]),
    );
    assert_eq!(e.code, ErrorCode::ShapeViolation);
    assert_eq!(
        e.expected,
        Some(json!({ "anchor": "Service", "rel": "type_of" }))
    );
    assert_eq!(e.actual, Some(json!("Invoice")));
    // The carrier is matched against the carried slot's pattern.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "send",
                    "source": { "node": "Payments", "port": "out" },
                    "carrier": "Payments",
                    "target": { "node": "Payments", "port": "recv" } })
        ),
        ErrorCode::ShapeViolation
    );
}

#[test]
fn e_carrier_required_and_forbidden() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "M" },
        { "stmt": "define", "conn": "plain", "directed": true, "source": "*", "target": "*" },
        { "stmt": "define", "conn": "send", "directed": true,
          "source": "*", "carrier": { "node": "M" }, "target": "*" }
    ]));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "send",
                    "source": { "node": "A", "port": "x" }, "target": { "node": "B", "port": "y" } })
        ),
        ErrorCode::CarrierRequired
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "plain",
                    "source": { "node": "A", "port": "x" }, "carrier": "M",
                    "target": { "node": "B", "port": "y" } })
        ),
        ErrorCode::CarrierForbidden
    );
}

#[test]
fn e_port_type_and_side_conflicts() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" },
        { "stmt": "define", "conn": "d", "directed": true, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "c",
          "source": { "node": "A", "port": "p" }, "target": { "node": "B", "port": "q" } }
    ]));
    let (_, e) = err(
        &mut ws,
        json!([{ "stmt": "conn-edge", "conn": "d",
                 "source": { "node": "A", "port": "p" }, "target": { "node": "B", "port": "r" } }]),
    );
    assert_eq!(e.code, ErrorCode::PortTypeConflict);
    // A.p was fixed to the source side; reusing it as a target is rejected.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "c",
                    "source": { "node": "B", "port": "out" }, "target": { "node": "A", "port": "p" } })
        ),
        ErrorCode::PortSideConflict
    );
    // An undirected type never fixes sides.
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "conn": "link", "directed": false, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "link",
          "source": { "node": "A", "port": "p" }, "target": { "node": "B", "port": "q" } }
    ]));
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "link",
                "source": { "node": "B", "port": "x" }, "target": { "node": "A", "port": "p" } })
    )));
}

#[test]
fn e_no_outer_port() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.Inner" }
    ]));
    // The outer port must exist (some connection attaches to it).
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "app", "node": "Orders", "port": "ghost",
                    "inner": { "node": "Inner", "port": "q" } })
        ),
        ErrorCode::NoOuterPort
    );
}

#[test]
fn e_ambiguous_delegation() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "Message" },
        { "stmt": "define", "node": "OrderCreated" },
        { "stmt": "define", "node": "PaymentFailed" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Message", "target": "OrderCreated" },
        { "stmt": "rel-edge", "rel": "type_of", "source": "Message", "target": "PaymentFailed" },
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "Orders" },
        { "stmt": "define", "node": "Orders.OrderHandler" },
        { "stmt": "define", "node": "Orders.PaymentHandler" },
        { "stmt": "define", "node": "Orders.Audit" },
        { "stmt": "define", "conn": "send", "directed": true,
          "source": "*", "carrier": { "anchor": "Message", "rel": "type_of" }, "target": "*" },
        { "stmt": "conn-edge", "conn": "send",
          "source": { "node": "A", "port": "out" }, "carrier": "OrderCreated",
          "target": { "node": "Orders", "port": "events" } },
        { "stmt": "app", "node": "Orders", "port": "events",
          "route": { "node": "OrderCreated" }, "inner": { "node": "OrderHandler", "port": "handle" } },
        { "stmt": "app", "node": "Orders", "port": "events",
          "inner": { "node": "Audit", "port": "all" } }
    ]));
    // A second unqualified delegation on the same port is ambiguous.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "app", "node": "Orders", "port": "events",
                    "inner": { "node": "PaymentHandler", "port": "all" } })
        ),
        ErrorCode::AmbiguousDelegation
    );
    // A qualified delegation whose pattern overlaps an existing qualifier
    // (both match OrderCreated) is ambiguous.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "app", "node": "Orders", "port": "events",
                    "route": { "anchor": "Message", "rel": "type_of" },
                    "inner": { "node": "PaymentHandler", "port": "handle" } })
        ),
        ErrorCode::AmbiguousDelegation
    );
    // Disjoint qualifiers coexist.
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "app", "node": "Orders", "port": "events",
                "route": { "node": "PaymentFailed" }, "inner": { "node": "PaymentHandler", "port": "handle" } })
    )));
}

#[test]
fn e_cross_scope_connection() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "node": "B.Inner" },
        { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" }
    ]));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "c",
                    "source": { "node": "A", "port": "x" }, "target": { "node": "B.Inner", "port": "y" } })
        ),
        ErrorCode::CrossScope
    );
}

#[test]
fn e_stdlib_protected() {
    let mut ws = Workspace::new();
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "delete", "rel": "type_of" })),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "redefine", "rel": "type_of", "trans": true, "directed": false,
                    "source": "*", "target": "*" })
        ),
        ErrorCode::StdlibProtected
    );
    // Restating the stdlib definition identically is a safe no-op — as a
    // define and as a redefine.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
                "source": "*", "target": "*" })
    )));
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "redefine", "rel": "type_of", "trans": true, "directed": true,
                "source": "*", "target": "*" })
    )));
    // A divergent define is an ordinary redeclaration error, without a
    // redefine hint: the redefine would be rejected too.
    let (_, e) = err(
        &mut ws,
        json!([{ "stmt": "define", "rel": "type_of", "trans": false, "directed": true,
                 "source": "*", "target": "*" }]),
    );
    assert_eq!(e.code, ErrorCode::Redeclared);
    assert!(e.hint.is_none(), "no hint towards a protected redefine");
}

#[test]
fn batches_are_atomic() {
    let mut ws = Workspace::new();
    let (index, e) = err(
        &mut ws,
        json!([
            { "stmt": "define", "node": "A" },
            { "stmt": "define", "node": "B" },
            { "stmt": "rel-edge", "rel": "nope", "source": "A", "target": "B" }
        ]),
    );
    assert_eq!(index, 2);
    assert_eq!(e.code, ErrorCode::UnknownName);
    assert_eq!(
        e.subject,
        Some(json!({ "stmt": "rel-edge", "rel": "nope", "source": "A", "target": "B" }))
    );
    // The whole batch rolled back: A and B were never created; no revision.
    assert!(ws.model().dump().is_empty());
    assert_eq!(ws.revision(), 0);
}

#[test]
fn a_parse_error_reports_its_index_and_rolls_back() {
    let mut ws = Workspace::new();
    let (index, e) = err(
        &mut ws,
        json!([
            { "stmt": "define", "node": "A" },
            { "stmt": "define" }
        ]),
    );
    assert_eq!(index, 1);
    assert_eq!(e.code, ErrorCode::Parse);
    assert!(ws.model().dump().is_empty());
}

#[test]
fn a_failed_statement_leaves_no_partial_state() {
    // The failing connection statement would create B.q2 before hitting the
    // side conflict on A.p; the batch rolls back as a whole, so q2 stays free
    // and can later bind a different type.
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "B" },
        { "stmt": "define", "conn": "c", "directed": true, "source": "*", "target": "*" },
        { "stmt": "define", "conn": "d", "directed": true, "source": "*", "target": "*" },
        { "stmt": "conn-edge", "conn": "c",
          "source": { "node": "A", "port": "p" }, "target": { "node": "B", "port": "q" } }
    ]));
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "conn-edge", "conn": "c",
                    "source": { "node": "B", "port": "q2" }, "target": { "node": "A", "port": "p" } })
        ),
        ErrorCode::PortSideConflict
    );
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "conn-edge", "conn": "d",
                "source": { "node": "B", "port": "q2" }, "target": { "node": "A", "port": "other" } })
    )));
}

// ---- envelope ---------------------------------------------------------------

#[test]
fn envelope_ok_and_revision() {
    let mut ws = Workspace::new();
    let r = ws.handle(&json!({
        "statements": [
            { "stmt": "define", "node": "A" },
            { "stmt": "check" }
        ]
    }));
    assert_eq!(r.status, "ok");
    assert_eq!(r.revision, 1);
    assert_eq!(r.results.as_ref().map(Vec::len), Some(2));
    // A noop-only request does not bump the revision.
    let r = ws.handle(&json!({
        "statements": [ { "stmt": "define", "node": "A" } ]
    }));
    assert_eq!(r.revision, 1);
}

#[test]
fn envelope_statement_error_carries_index() {
    let mut ws = Workspace::new();
    let r = ws.handle(&json!({
        "statements": [
            { "stmt": "define", "node": "A" },
            { "stmt": "rel-edge", "rel": "nope", "source": "A", "target": "A" }
        ]
    }));
    assert_eq!(r.status, "error");
    let err = r.error.expect("error present");
    assert_eq!(err.index, Some(1));
    assert_eq!(err.error.code, ErrorCode::UnknownName);
    assert_eq!(r.revision, 0, "the batch rolled back");
}

#[test]
fn envelope_protocol_errors() {
    let mut ws = Workspace::new();
    let r = ws.handle(&json!({ "statement": [] }));
    assert_eq!(r.status, "error");
    let err = r.error.expect("error present");
    assert_eq!(err.error.code, ErrorCode::BadRequest);
    assert_eq!(err.index, None);

    let r = ws.handle(&json!({ "statements": [], "expect_revision": 9 }));
    assert_eq!(r.error.expect("stale").error.code, ErrorCode::StaleRevision);
}

#[test]
fn envelope_dry_run_previews_without_applying() {
    let mut ws = ws_with(json!([
        { "stmt": "define", "node": "A" },
        { "stmt": "define", "node": "A.Inner" }
    ]));
    let before = ws.revision();
    let r = ws.handle(&json!({
        "statements": [ { "stmt": "delete", "node": "A" } ],
        "dry_run": true
    }));
    assert_eq!(r.status, "ok");
    assert_eq!(r.revision, before);
    let results = r.results.expect("results");
    match &results[0] {
        modeling_lang::Outcome::Applied { cascade: Some(c) } => {
            assert_eq!(c.len(), 2, "the cascade preview covers A and A.Inner");
        }
        o => panic!("expected a cascade preview, got {o:?}"),
    }
    // Nothing was applied.
    assert!(ws.model().layer_of("A").is_some());
}
