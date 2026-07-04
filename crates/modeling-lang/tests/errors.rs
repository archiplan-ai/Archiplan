//! One test per code in the error catalog, plus the atomicity guarantees of
//! the statement API (`requirements/modeling-lang/errors.md`).

mod common;

use common::*;
use modeling_lang::{ErrorCode, Session};

#[test]
fn e_parse_reports_position_and_subject() {
    let mut s = Session::new();
    let e = err(&mut s, "node ;");
    assert_eq!(e.code, ErrorCode::Parse);
    assert!(
        e.message.contains("at 1:"),
        "position included: {}",
        e.message
    );
    assert!(!e.subject.is_empty());
}

#[test]
fn e_parse_missing_separator() {
    let mut s = Session::new();
    assert_eq!(err_code(&mut s, "node A node B"), ErrorCode::Parse);
}

#[test]
fn e_parse_views_on_application() {
    let mut s = session_with("node A; node B; conn c := * -> *; A(x) c B(y);");
    let e = err(&mut s, "B { y = missing(z) in nope; }");
    assert_eq!(e.code, ErrorCode::Parse);
}

#[test]
fn e_unknown_name_node_rel_view() {
    let mut s = session_with("node A; node B;");
    assert_eq!(err_code(&mut s, "Ghost type_of A;"), ErrorCode::UnknownName);
    assert_eq!(err_code(&mut s, "A nope B;"), ErrorCode::UnknownName);
    assert_eq!(
        err_code(&mut s, "A type_of B in nope;"),
        ErrorCode::UnknownName
    );
    assert_eq!(err_code(&mut s, "open Ghost;"), ErrorCode::UnknownName);
    assert_eq!(err_code(&mut s, "delete Ghost;"), ErrorCode::UnknownName);
}

#[test]
fn e_unknown_name_for_a_missing_edge() {
    let mut s = session_with("node A; node B; rel r := * -> *;");
    assert_eq!(err_code(&mut s, "delete A r B;"), ErrorCode::UnknownName);
    assert_eq!(
        err_code(&mut s, "untag A r B in v;"),
        ErrorCode::UnknownName
    );
}

#[test]
fn e_dup_name_on_rename() {
    let mut s = session_with("node A; node B;");
    let e = err(&mut s, "rename A B;");
    assert_eq!(e.code, ErrorCode::DupName);
    // Renaming to its own name is a no-op, not a collision.
    assert!(is_noop(&outcome(&mut s, "rename A A;")));
}

#[test]
fn e_redeclared_with_existing_definition_included() {
    let mut s = session_with("rel r := * -> *;");
    assert!(is_noop(&outcome(&mut s, "rel r := * -> *;")));
    let e = err(&mut s, "rel r := * <-> *;");
    assert_eq!(e.code, ErrorCode::Redeclared);
    assert_eq!(e.actual.as_deref(), Some("rel r := * -> *;"));
    // One namespace for edge types: a conn cannot reuse a rel name.
    assert_eq!(err_code(&mut s, "conn r := * -> *;"), ErrorCode::Redeclared);
}

#[test]
fn e_shape_violation_on_ends_and_carrier() {
    let mut s = session_with(
        "node Service; node Data; node Payments; node Invoice;
         Service type_of Payments; Data type_of Invoice;
         conn calls := (Service type_of *) -> (Service type_of *);
         conn send := (Service type_of *) (Data type_of *)-> (Service type_of *);",
    );
    let e = err(&mut s, "Payments(out) calls Invoice(recv);");
    assert_eq!(e.code, ErrorCode::ShapeViolation);
    assert_eq!(e.expected.as_deref(), Some("(Service type_of *)"));
    assert_eq!(e.actual.as_deref(), Some("Invoice"));
    // The carrier is matched against the carried slot's pattern.
    assert_eq!(
        err_code(&mut s, "Payments(out) send(Payments) Payments(recv);"),
        ErrorCode::ShapeViolation
    );
    // Relations check shapes too.
    outcomes(
        &mut s,
        "rel stores := (Service type_of *) -> (Data type_of *);",
    );
    assert_eq!(
        err_code(&mut s, "Payments stores Payments;"),
        ErrorCode::ShapeViolation
    );
}

#[test]
fn e_carrier_required_and_forbidden() {
    let mut s = session_with(
        "node A; node B; node M;
         conn plain := * -> *;
         conn send := * (M)-> *;",
    );
    assert_eq!(
        err_code(&mut s, "A(x) send B(y);"),
        ErrorCode::CarrierRequired
    );
    assert_eq!(
        err_code(&mut s, "A(x) plain(M) B(y);"),
        ErrorCode::CarrierForbidden
    );
}

#[test]
fn e_port_type_conflict() {
    let mut s = session_with(
        "node A; node B;
         conn c := * -> *;
         conn d := * -> *;
         A(p) c B(q);",
    );
    let e = err(&mut s, "A(p) d B(r);");
    assert_eq!(e.code, ErrorCode::PortTypeConflict);
    assert_eq!(e.expected.as_deref(), Some("c"));
    assert_eq!(e.actual.as_deref(), Some("d"));
}

#[test]
fn e_port_side_conflict() {
    let mut s = session_with(
        "node A; node B;
         conn c := * -> *;
         A(p) c B(q);",
    );
    // A.p was fixed to the source side; reusing it as a target is rejected.
    assert_eq!(
        err_code(&mut s, "B(out) c A(p);"),
        ErrorCode::PortSideConflict
    );
    // An undirected type never fixes sides.
    let mut s = session_with(
        "node A; node B;
         conn link := * <-> *;
         A(p) link B(q);",
    );
    assert!(is_applied(&outcome(&mut s, "B(x) link A(p);")));
}

#[test]
fn e_no_outer_port() {
    let mut s = session_with("node Orders; node H;");
    // At the root there is no enclosing node whose port could be delegated.
    assert_eq!(err_code(&mut s, "p = H(q);"), ErrorCode::NoOuterPort);
    // Inside a node, the outer port must exist (some connection attaches it).
    assert_eq!(
        err_code(&mut s, "Orders { node Inner; ghost = Inner(q); }"),
        ErrorCode::NoOuterPort
    );
}

#[test]
fn e_ambiguous_delegation() {
    let mut s = session_with(
        "node Message; node OrderCreated; node PaymentFailed;
         Message type_of OrderCreated; Message type_of PaymentFailed;
         node A; node Orders;
         conn send := * (Message type_of *)-> *;
         A(out) send(OrderCreated) Orders(events);
         node Orders {
           node OrderHandler; node PaymentHandler; node Audit;
           events(OrderCreated) = OrderHandler(handle);
           events = Audit(all);
         }",
    );
    // A second unqualified delegation on the same port is ambiguous.
    assert_eq!(
        err_code(&mut s, "Orders { node Audit2; events = Audit2(all); }"),
        ErrorCode::AmbiguousDelegation
    );
    // A qualified delegation whose pattern overlaps an existing qualifier
    // (both match OrderCreated) is ambiguous.
    assert_eq!(
        err_code(
            &mut s,
            "Orders { events(Message type_of *) = PaymentHandler(handle); }"
        ),
        ErrorCode::AmbiguousDelegation
    );
    // Disjoint qualifiers coexist.
    assert!(matches!(
        outcome(
            &mut s,
            "Orders { events(PaymentFailed) = PaymentHandler(handle); }"
        ),
        modeling_lang::Outcome::Block(_)
    ));
}

#[test]
fn e_stdlib_protected() {
    let mut s = Session::new();
    assert_eq!(
        err_code(&mut s, "delete rel type_of;"),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(&mut s, "rel type_of := * <-> *;"),
        ErrorCode::StdlibProtected
    );
    // Restating the stdlib definition identically is a safe no-op.
    assert!(is_noop(&outcome(&mut s, "rel trans type_of := * -> *;")));
}

#[test]
fn e_cross_scope_connection() {
    let mut s = session_with(
        "node A; node B { node Inner; }
         conn c := * -> *;",
    );
    assert_eq!(
        err_code(&mut s, "A(x) c B.Inner(y);"),
        ErrorCode::CrossScope
    );
}

#[test]
fn e_cross_scope_application_skips_a_boundary() {
    let mut s = session_with(
        "node Orders { node Sub { node Deep; } }
         node X;
         conn c := * -> *;
         X(out) c Orders(p);",
    );
    assert_eq!(
        err_code(&mut s, "Orders { p = Sub.Deep(q); }"),
        ErrorCode::CrossScope
    );
}

#[test]
fn batch_is_atomic() {
    let mut s = Session::new();
    let b = match s.execute("node A; node B; A nope B;") {
        Err(b) => b,
        Ok(_) => panic!("expected failure"),
    };
    assert_eq!(b.index, 2);
    assert_eq!(b.error.code, ErrorCode::UnknownName);
    assert_eq!(b.error.subject, "A nope B");
    // The whole batch rolled back: A and B were never created.
    assert_eq!(err_code(&mut s, "open A;"), ErrorCode::UnknownName);
    assert!(s.model().dump().is_empty());
}

#[test]
fn interactive_statements_apply_one_at_a_time() {
    let mut s = Session::new();
    let results = s.execute_interactive("node A; node B; A nope B; node C;");
    assert_eq!(results.len(), 4);
    assert!(results[0].result.is_ok());
    assert!(results[1].result.is_ok());
    assert!(results[2].result.is_err());
    assert!(
        results[3].result.is_ok(),
        "an error does not stop later statements"
    );
    assert_eq!(s.model().dump(), vec!["node A;", "node B;", "node C;"]);
}

#[test]
fn a_failed_statement_leaves_no_partial_state() {
    // The failing connection statement would create B.q2 before hitting the
    // side conflict on A.p; the statement rolls back as a whole, so q2 stays
    // free and can later bind a different type.
    let mut s = session_with(
        "node A; node B;
         conn c := * -> *;
         conn d := * -> *;
         A(p) c B(q);",
    );
    assert_eq!(
        err_code(&mut s, "B(q2) c A(p);"),
        ErrorCode::PortSideConflict
    );
    assert!(is_applied(&outcome(&mut s, "B(q2) d A(other);")));
}

#[test]
fn a_failed_block_rolls_back_entirely() {
    let mut s = session_with("node Orders;");
    let e = err(&mut s, "Orders { node H; ghost = H(q); }");
    assert_eq!(e.code, ErrorCode::NoOuterPort);
    assert_eq!(e.subject, "ghost = H(q)");
    // The inner `node H` that had already applied inside the block is gone.
    assert_eq!(s.model().dump(), vec!["node Orders;"]);
}
