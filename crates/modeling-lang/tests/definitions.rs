//! Element definitions (`archi/requirements/element-definitions/`): comments
//! in definition position attach as identity prose, the prose gate rejects
//! obligations through every door, and definitions ride the lowered
//! statements.

use std::collections::BTreeMap;

use modeling_lang::source::{Compiled, compile_project, compile_sources};
use modeling_lang::{Definition, Preset, Statement, parse_statement};
use serde_json::json;
use std::path::Path;

fn compile(sources: &[(&str, &str)]) -> Compiled {
    match compile_sources(&Preset::default_ontology(), sources) {
        Ok(c) => c,
        Err(f) => panic!("compile failed:\n{}", f.render()),
    }
}

fn compile_errs(sources: &[(&str, &str)]) -> Vec<String> {
    match compile_sources(&Preset::default_ontology(), sources) {
        Ok(_) => panic!("expected a failure"),
        Err(f) => f.diagnostics.iter().map(|d| d.render(&f.map)).collect(),
    }
}

/// The docs of every define in the batch, keyed by subject path/name.
fn docs_of(c: &Compiled) -> BTreeMap<String, Option<String>> {
    let mut out = BTreeMap::new();
    for s in &c.batch {
        if let Statement::Define(d) = s {
            let (key, doc) = match d {
                Definition::Node { path, doc, .. } => (path.clone(), doc.clone()),
                Definition::View { name, doc } => (format!("view:{name}"), doc.clone()),
                Definition::Rel { name, doc, .. } => (format!("rel:{name}"), doc.clone()),
                Definition::Conn { name, doc, .. } => (format!("conn:{name}"), doc.clone()),
            };
            out.insert(key, doc);
        }
    }
    out
}

fn port_docs_of(c: &Compiled, node: &str) -> Option<BTreeMap<String, String>> {
    c.batch.iter().find_map(|s| match s {
        Statement::Define(Definition::Node {
            path, port_docs, ..
        }) if path == node => port_docs.clone(),
        _ => None,
    })
}

#[test]
fn definitions_attach_from_both_positions_on_every_element_kind() {
    let src = "\
def view flow // the story of one request
def rel owns := * -> * // ownership between components
// carries the login payload
def conn login := * ->Payload *
def node Payload // the login payload record
// The auth boundary:
// every credential crosses here.
def node Auth: // hmm replaced below
  port check // credentials in, verdict out
";
    // The node above claims both forms — split that case out; here use one.
    let src = src.replace("def node Auth: // hmm replaced below", "def node Auth:");
    let c = compile(&[("m", &src)]);
    let docs = docs_of(&c);
    assert_eq!(docs["view:flow"].as_deref(), Some("the story of one request"));
    assert_eq!(
        docs["rel:owns"].as_deref(),
        Some("ownership between components")
    );
    assert_eq!(
        docs["conn:login"].as_deref(),
        Some("carries the login payload")
    );
    assert_eq!(docs["Payload"].as_deref(), Some("the login payload record"));
    assert_eq!(
        docs["Auth"].as_deref(),
        Some("The auth boundary: every credential crosses here.")
    );
    assert_eq!(
        port_docs_of(&c, "Auth").unwrap()["check"],
        "credentials in, verdict out"
    );
}

#[test]
fn blank_lines_detach_and_non_defining_lines_take_nothing() {
    let src = "\
// a file header, not a definition

def node A

// section prose, detached

def node B:
  port x
// commentary above an open, not a definition
open B:
  def node Inner
";
    let c = compile(&[("m", src)]);
    let docs = docs_of(&c);
    assert_eq!(docs["A"], None);
    assert_eq!(docs["B"], None);
    assert_eq!(docs["B.Inner"], None);
}

#[test]
fn whitespace_only_comments_attach_nothing() {
    let src = "def node A: //\n  port x //   \n";
    let c = compile(&[("m", src)]);
    assert_eq!(docs_of(&c)["A"], None);
    assert_eq!(port_docs_of(&c, "A"), None);
}

#[test]
fn claiming_both_forms_is_one_located_error() {
    let src = "// from above\ndef node A // and trailing\n";
    let errs = compile_errs(&[("m", src)]);
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0].contains("E_DEFINITION") && errs[0].contains("keep one"),
        "{}",
        errs[0]
    );
    assert!(errs[0].contains("m.arch:2:"), "{}", errs[0]);
}

#[test]
fn violations_locate_the_comment_and_all_surface_in_one_pass() {
    let src = "\
def node A // must reject tabs
def node B // one thing. another thing
";
    let errs = compile_errs(&[("m", src)]);
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert!(
        errs[0].contains("m.arch:1:12") && errs[0].contains("obligation"),
        "{}",
        errs[0]
    );
    assert!(
        errs[1].contains("m.arch:2:12") && errs[1].contains("single sentence"),
        "{}",
        errs[1]
    );
}

#[test]
fn block_definitions_normalize_across_lines() {
    let src = "\
//   the   auth
//  boundary of the   system
def node A
";
    let c = compile(&[("m", src)]);
    assert_eq!(
        docs_of(&c)["A"].as_deref(),
        Some("the auth boundary of the system")
    );
}

#[test]
fn the_obligation_gate_rejects_every_splice_and_none() {
    for bad in [
        "must reject tabs",
        "the tokenizer, must reject tabs",
        "the tokenizer; must reject tabs",
        "the tokenizer — must reject tabs",
        "the tokenizer: ensures determinism",
        "Handles the login flow",
    ] {
        let src = format!("def node A // {bad}\n");
        let errs = compile_errs(&[("m", &src)]);
        assert!(
            errs[0].contains("obligation"),
            "`{bad}` should reject: {}",
            errs[0]
        );
    }
    for ok in ["a mustard-colored handler", "the marshall of tokens"] {
        let src = format!("def node A // {ok}\n");
        compile(&[("m", &src)]);
    }
}

#[test]
fn the_statement_api_meets_the_same_gate() {
    // Valid docs pass and normalize.
    let s = parse_statement(&json!({
        "stmt": "define", "node": "A", "ports": ["x"],
        "doc": "  the   auth  boundary ", "port_docs": {"x": "credentials in"}
    }))
    .expect("valid define");
    let Statement::Define(Definition::Node { doc, port_docs, .. }) = &s else {
        panic!("a define");
    };
    assert_eq!(doc.as_deref(), Some("the auth boundary"));
    assert_eq!(port_docs.as_ref().unwrap()["x"], "credentials in");

    // The same prose the source gate rejects, rejected here.
    let e = parse_statement(&json!({
        "stmt": "define", "node": "A", "doc": "must reject tabs"
    }))
    .expect_err("obligation prose");
    assert!(e.message.contains("obligation"), "{}", e.message);

    // port_docs is anchored to declared ports.
    let e = parse_statement(&json!({
        "stmt": "define", "node": "A", "port_docs": {"x": "text"}
    }))
    .expect_err("port_docs without ports");
    assert!(e.message.contains("requires `ports`"), "{}", e.message);
    let e = parse_statement(&json!({
        "stmt": "define", "node": "A", "ports": ["y"], "port_docs": {"x": "text"}
    }))
    .expect_err("port_docs names an undeclared port");
    assert!(e.message.contains("not in `ports`"), "{}", e.message);

    // An unknown sibling field still rejects — the schema stays strict.
    let e = parse_statement(&json!({
        "stmt": "define", "node": "A", "docs": "typo"
    }))
    .expect_err("unknown field");
    assert!(e.message.contains("unknown field"), "{}", e.message);
}

#[test]
fn lowered_statements_carry_definitions_in_pseudo_and_replay() {
    let src = "\
def node Agent: // the operator outside the tool
  port drive // runs the verbs
";
    let c = compile(&[("m", src)]);
    let stmt = c
        .batch
        .iter()
        .find(|s| matches!(s, Statement::Define(Definition::Node { path, .. }) if path == "Agent"))
        .expect("the Agent define");
    let pseudo = stmt.pseudo();
    assert_eq!(
        pseudo,
        "def node Agent: // the operator outside the tool\n  port drive // runs the verbs"
    );
    // The pseudo text is valid source: recompiling it reproduces the docs.
    let again = compile(&[("m", &format!("{pseudo}\n"))]);
    assert_eq!(
        docs_of(&again)["Agent"].as_deref(),
        Some("the operator outside the tool")
    );
    assert_eq!(port_docs_of(&again, "Agent").unwrap()["drive"], "runs the verbs");
}

#[test]
fn definitions_join_element_identity_in_the_engine() {
    use modeling_lang::Workspace;
    let mut ws = Workspace::new();
    ws.execute_values(&[
        json!({"stmt": "define", "node": "A", "ports": ["p"],
               "doc": "the auth boundary", "port_docs": {"p": "credentials in"}}),
        json!({"stmt": "define", "view": "flow", "doc": "one story"}),
        json!({"stmt": "define", "rel": "owns", "directed": true,
               "source": "*", "target": "*", "doc": "ownership"}),
    ])
    .expect("the batch applies");

    // Identical restatements no-op; omitted docs make no claim.
    let outcomes = ws
        .execute_values(&[
            json!({"stmt": "define", "node": "A", "ports": ["p"],
                   "doc": "the auth boundary", "port_docs": {"p": "credentials in"}}),
            json!({"stmt": "define", "node": "A"}),
            json!({"stmt": "define", "view": "flow"}),
            json!({"stmt": "define", "rel": "owns", "directed": true,
                   "source": "*", "target": "*"}),
        ])
        .expect("restatements apply");
    assert!(
        outcomes
            .iter()
            .all(|o| matches!(o, modeling_lang::Outcome::Noop)),
        "{outcomes:?}"
    );

    // Divergent definitions reject like divergent ports.
    for (divergent, what) in [
        (
            json!({"stmt": "define", "node": "A", "doc": "something else"}),
            "node doc",
        ),
        (
            json!({"stmt": "define", "node": "A", "ports": ["p"],
                   "port_docs": {"p": "something else"}}),
            "port doc",
        ),
        (
            json!({"stmt": "define", "view": "flow", "doc": "another story"}),
            "view doc",
        ),
        (
            json!({"stmt": "define", "rel": "owns", "directed": true,
                   "source": "*", "target": "*", "doc": "something else"}),
            "rel doc",
        ),
    ] {
        let e = ws.execute_values(&[divergent]).expect_err(what).error;
        assert_eq!(e.code.as_str(), "E_REDECLARED", "{what}: {}", e.message);
    }
}

#[test]
fn the_execute_door_meets_the_same_gate() {
    use modeling_lang::Workspace;
    let mut ws = Workspace::new();
    // Obligation prose rejects through direct execute — no JSON schema ran.
    let stmt = Statement::Define(Definition::Node {
        path: "A".into(),
        ports: None,
        doc: Some("must reject tabs".into()),
        port_docs: None,
    });
    let e = ws.execute(&[stmt]).expect_err("obligation prose").error;
    assert_eq!(e.code.as_str(), "E_PARSE");
    assert!(e.message.contains("obligation"), "{}", e.message);

    // Unnormalized text normalizes at the door, so a raw restatement of a
    // normalized store is still the identical claim.
    let raw = Statement::Define(Definition::Node {
        path: "B".into(),
        ports: None,
        doc: Some("  the   auth\tboundary ".into()),
        port_docs: None,
    });
    ws.execute(&[raw]).expect("normalizes and applies");
    ws.execute_values(&[json!({"stmt": "define", "node": "B", "doc": "the auth boundary"})])
        .expect("the normalized restatement is identical");

    // port_docs on a port outside the declared set rejects at this door too.
    let stray = Statement::Define(Definition::Node {
        path: "C".into(),
        ports: Some(vec!["x".into()]),
        doc: None,
        port_docs: Some([("y".to_string(), "text".to_string())].into()),
    });
    let e = ws.execute(&[stray]).expect_err("undeclared port").error;
    assert!(e.message.contains("not in `ports`"), "{}", e.message);
}

#[test]
fn the_canonical_render_carries_definitions_byte_stably() {
    let src = "\
def view flow // the story of one request
def rel owns := * -> * // ownership between components
// carries the login payload
def conn login := * ->Payload *
def node Payload // the login payload record
def node Auth: // the auth boundary
  port check // credentials in, verdict out
Auth.check login(Payload) Auth.check in flow
";
    let c = compile(&[("m", src)]);
    let render = c.workspace.model().render_source();
    assert!(
        render.contains("def node Payload // the login payload record"),
        "{render}"
    );
    assert!(render.contains("  port check // credentials in, verdict out"));
    assert!(render.contains("def view flow // the story of one request"));
    assert!(render.contains("// carries the login payload"));

    // Compiling the render reproduces the model and the bytes.
    let again = compile(&[("model", &render)]);
    assert_eq!(docs_of(&again), docs_of(&c));
    assert_eq!(again.workspace.model().render_source(), render);
}

#[test]
fn renders_stay_layout_blind_with_definitions() {
    let one = compile(&[(
        "all",
        "def node A: // alpha\n  port p // in\ndef node B // beta\nA dep_on B\ndef rel dep_on := * -> * // dependency\n",
    )]);
    let two = compile(&[
        ("zz_nodes", "import types\nimport zz_more\ndef node A: // alpha\n  port p // in\nA dep_on B\n"),
        ("zz_more", "def node B // beta\n"),
        ("types", "def rel dep_on := * -> * // dependency\n"),
    ]);
    assert_eq!(
        one.workspace.model().render_source(),
        two.workspace.model().render_source()
    );
}

#[test]
fn this_repository_reads_its_own_definitions() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let c = match compile_project(root) {
        Ok(c) => c,
        Err(f) => panic!("the repo model failed to compile:\n{}", f.render()),
    };
    let docs = docs_of(&c);
    assert_eq!(port_docs_of(&c, "Agent").unwrap()["drive"], "runs the verbs");
    assert_eq!(
        docs["Compiler.Definitions"].as_deref(),
        Some("the prose gate: attaches each definition comment to its element and rejects obligation prose")
    );
    assert_eq!(
        docs["view:compile_flow"].as_deref(),
        Some("source to model: the compiler pipeline and the engine")
    );
    assert_eq!(
        docs["conn:annotate"].as_deref(),
        Some("definitions attached and validated")
    );
}
