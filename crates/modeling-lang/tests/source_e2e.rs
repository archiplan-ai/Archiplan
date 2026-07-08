//! End-to-end compilation of `.arch` projects: the auth fixture, queries and
//! NKP over the compiled model, determinism, engine-error localization, and
//! engine-level round-trips.

use modeling_lang::source::{Compiled, compile_project, compile_sources};
use modeling_lang::{NkpConfig, Outcome, Preset, Workspace};
use serde_json::json;
use std::path::Path;

fn fixture_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/auth"))
}

fn compile_fixture() -> Compiled {
    match compile_project(fixture_root()) {
        Ok(c) => c,
        Err(f) => panic!("fixture failed to compile:\n{}", f.render()),
    }
}

#[test]
fn the_auth_fixture_compiles_and_answers_queries() {
    let mut compiled = compile_fixture();
    // The login_flow slice: UI wired to AuthService through the bidir conn.
    let results = compiled
        .workspace
        .execute_values(
            json!([{ "stmt": "query", "views": ["login_flow"] }])
                .as_array()
                .unwrap(),
        )
        .expect("query runs");
    let Outcome::Graph { nodes, edges } = &results[0] else {
        panic!("expected a graph");
    };
    let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(
        ids.contains(&"UI") && ids.contains(&"AuthService"),
        "{ids:?}"
    );
    // The conn edge plus the delegation that routes it — an application
    // belongs to the views of the edges it routes.
    let values: Vec<serde_json::Value> = edges
        .iter()
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();
    assert_eq!(values.len(), 2, "{values:?}");
    let conn = values.iter().find(|e| e["kind"] == "connection").unwrap();
    assert_eq!(conn["type"], "login");
    assert_eq!(conn["carrier"], "LoginForm");
    assert_eq!(conn["rev_carrier"], "AuthResponse");
    assert_eq!(conn["source_port"], "login");
    assert_eq!(conn["target_port"], "handle_login");
    assert!(values.iter().any(|e| e["kind"] == "application"));

    // The internals: delegation and the store edge inside AuthService.
    let results = compiled
        .workspace
        .execute_values(
            json!([{ "stmt": "query", "scopes": ["AuthService"] }])
                .as_array()
                .unwrap(),
        )
        .expect("query runs");
    let Outcome::Graph { nodes, edges } = &results[0] else {
        panic!("expected a graph");
    };
    let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains(&"AuthService.Storage"), "{ids:?}");
    assert!(ids.contains(&"AuthService.LoginHandler"), "{ids:?}");
    assert!(
        edges
            .iter()
            .any(|e| serde_json::to_value(e).unwrap()["kind"] == "application"),
        "the handle_login delegation is part of the slice"
    );
}

#[test]
fn the_fixture_passes_nkp_and_check() {
    let compiled = compile_fixture();
    let report = compiled
        .workspace
        .model()
        .nkp(&NkpConfig::default())
        .expect("nkp runs on the compiled model");
    assert!(!report.matrix.nodes.is_empty());
    // Interface-first construction leaves declared-but-unwired ports; that is
    // a finding, never an error.
    let findings = compiled.workspace.model().check();
    assert!(
        findings
            .iter()
            .all(|f| matches!(f, modeling_lang::Finding::UnusedPort { .. })),
        "only unused-port findings expected: {findings:?}"
    );
}

#[test]
fn compilation_is_deterministic_under_source_order() {
    let sources = [
        (
            "auth",
            "def node AuthService:\n  port handle_login\nService type_of AuthService\n",
        ),
        (
            "ui",
            "import auth\nimport conns\ndef view login_flow\ndef node UI:\n  port login\nUI.login login AuthService.handle_login in login_flow\n",
        ),
        (
            "conns",
            "import messages\ndef conn login := * ->LoginForm, <-AuthResponse *\n",
        ),
        ("messages", "def node LoginForm\ndef node AuthResponse\n"),
    ];
    let preset = Preset::default_ontology();
    let a = compile_sources(&preset, &sources)
        .map_err(|f| f.render())
        .unwrap();
    let mut permuted = sources;
    permuted.reverse();
    let b = compile_sources(&preset, &permuted)
        .map_err(|f| f.render())
        .unwrap();
    let batch_a: Vec<_> = a.batch.iter().map(|s| s.to_value()).collect();
    let batch_b: Vec<_> = b.batch.iter().map(|s| s.to_value()).collect();
    assert_eq!(batch_a, batch_b);
}

#[test]
fn compilation_is_invariant_under_module_renaming() {
    // Same model, module names shuffled so BOTH edge-carrying modules flip
    // across the sort boundary — the texture
    // `compilation_is_deterministic_under_source_order` cannot reach, since
    // permuting discovery order never permutes names
    // (`issues/carrier-inference-order-dependence.md`,
    // `issues/canonical-render-edge-order-depends-on-module-names.md`).
    // Only module names and import lines differ; both spellings must lower
    // to a bit-identical batch and render to identical bytes — the batch is
    // a function of the model, never of file layout.
    let preset = Preset::default_ontology();
    let late_use = [
        (
            "aconns",
            "import mmsgs\ndef conn login := * ->LoginForm, <-AuthResponse *\n",
        ),
        ("mmsgs", "def node LoginForm\ndef node AuthResponse\n"),
        (
            "zui",
            "import aconns\ndef view login_flow\ndef node UI:\n  port login\ndef node AuthService:\n  port handle_login\nUI.login login AuthService.handle_login in login_flow\n",
        ),
        (
            "zzops",
            "import zui\nimport aconns\ndef node Operator:\n  port drive\nOperator.drive login AuthService.handle_login in login_flow\nService type_of Operator\n",
        ),
    ];
    let early_use = [
        (
            "zconns",
            "import mmsgs\ndef conn login := * ->LoginForm, <-AuthResponse *\n",
        ),
        ("mmsgs", "def node LoginForm\ndef node AuthResponse\n"),
        (
            "xui",
            "import zconns\ndef view login_flow\ndef node UI:\n  port login\ndef node AuthService:\n  port handle_login\nUI.login login AuthService.handle_login in login_flow\n",
        ),
        (
            "aops",
            "import xui\nimport zconns\ndef node Operator:\n  port drive\nOperator.drive login AuthService.handle_login in login_flow\nService type_of Operator\n",
        ),
    ];
    let a = compile_sources(&preset, &late_use)
        .map_err(|f| f.render())
        .unwrap();
    let b = compile_sources(&preset, &early_use)
        .map_err(|f| f.render())
        .unwrap();
    let batch_a: Vec<_> = a.batch.iter().map(|s| s.to_value()).collect();
    let batch_b: Vec<_> = b.batch.iter().map(|s| s.to_value()).collect();
    assert_eq!(batch_a, batch_b);
    assert_eq!(
        a.workspace.model().render_source(),
        b.workspace.model().render_source()
    );
}

#[test]
fn delegation_chains_lower_outward_in_whatever_the_module_names() {
    // A delegation chain split across modules so the INNER application's
    // module sorts first: authoring order would replay the chained
    // application before the application that attaches its outer port and
    // the engine would refuse (`NoOuterPort`). Chain-ordered lowering
    // sequences applications outward-in regardless of file layout
    // (`issues/canonical-render-edge-order-depends-on-module-names.md`).
    let preset = Preset::default_ontology();
    let sources = [
        ("aainner", "import zouter\nGate.Desk.answer = Clerk.reply\n"),
        (
            "zouter",
            "def conn ask := * -> *\n\
             def node Gate:\n  port answer\n\
             \x20 def node Desk:\n    port answer\n\
             \x20   def node Clerk:\n      port reply\n\
             def node Caller:\n  port ask\n\
             Caller.ask ask Gate.answer\n\
             Gate.answer = Desk.answer\n",
        ),
    ];
    let compiled = compile_sources(&preset, &sources)
        .map_err(|f| f.render())
        .unwrap();
    let apps: Vec<String> = compiled
        .batch
        .iter()
        .filter_map(|s| {
            let v = s.to_value();
            (v["stmt"] == "app").then(|| v["node"].as_str().unwrap().to_string())
        })
        .collect();
    assert_eq!(
        apps,
        vec!["Gate".to_string(), "Gate.Desk".to_string()],
        "the outer application lowers before the chained inner one"
    );
}

#[test]
fn classifier_edges_land_before_shapes_that_consult_them() {
    // The classifying rel edge is written AFTER the conn edge that relies on
    // it; deterministic lowering must still order it first.
    let preset = Preset::default_ontology();
    let compiled = compile_sources(
        &preset,
        &[(
            "main",
            "def node A:\n  port out\ndef node B:\n  port inbox\n\
             def conn calls := (Service type_of *) -> (Service type_of *)\n\
             A.out calls B.inbox\n\
             Service type_of A\nService type_of B\n",
        )],
    )
    .map_err(|f| f.render())
    .expect("shape satisfied by later-written classifiers");
    assert!(!compiled.workspace.model().dump().is_empty());
}

#[test]
fn engine_errors_localize_to_source_lines() {
    // A connection may not cross a scope boundary; the engine rejects it and
    // the compiler points at the offending line.
    let preset = Preset::default_ontology();
    let err = compile_sources(
        &preset,
        &[(
            "main",
            "def node A:\n  port out\ndef node B:\n  def node Inner:\n    port in_p\ndef conn c := * -> *\nA.out c B.Inner.in_p\n",
        )],
    )
    .err()
    .expect("cross-scope connection is rejected");
    assert_eq!(err.diagnostics.len(), 1);
    let d = &err.diagnostics[0];
    assert_eq!(d.code, "E_CROSS_SCOPE");
    let (file, line, _col) = err
        .map
        .location(d.span.expect("engine errors are localized"));
    assert_eq!((file, line), ("src/main.arch", 7));
}

#[test]
fn rel_reference_cycles_are_def_cycle_errors() {
    let preset = Preset::default_ontology();
    let err = compile_sources(
        &preset,
        &[(
            "main",
            "def node X\ndef rel a := (X b *) -> *\ndef rel b := (X a *) -> *\n",
        )],
    )
    .err()
    .expect("cyclic rel shapes cannot be ordered");
    assert!(err.diagnostics.iter().all(|d| d.code == "E_DEF_CYCLE"));
    assert_eq!(err.diagnostics.len(), 2);
}

#[test]
fn parse_errors_from_several_files_are_collected() {
    let preset = Preset::default_ontology();
    let err = compile_sources(
        &preset,
        &[
            ("bad_a", "def node A:\n\tport x\n"),
            ("bad_b", "def view port\n"),
            ("good", "def node C\n"),
        ],
    )
    .err()
    .expect("parse failures");
    assert_eq!(err.diagnostics.len(), 2);
    assert!(err.diagnostics.iter().all(|d| d.code == "E_PARSE"));
}

#[test]
fn dumps_are_valid_surface_and_round_trip() {
    // Compile the fixture, render its dump as `.arch` text, compile THAT as
    // a single-module project: the models must be identical.
    let compiled = compile_fixture();
    let dump_text = compiled
        .workspace
        .model()
        .dump()
        .iter()
        .map(|s| s.pseudo())
        .collect::<Vec<_>>()
        .join("\n");
    let preset = Preset::default_ontology();
    let recompiled = match compile_sources(&preset, &[("dump", &dump_text)]) {
        Ok(c) => c,
        Err(f) => panic!(
            "dump is not valid surface:\n{}\n--- dump ---\n{dump_text}",
            f.render()
        ),
    };
    let a: Vec<String> = compiled
        .workspace
        .model()
        .dump()
        .iter()
        .map(|s| s.pseudo())
        .collect();
    let b: Vec<String> = recompiled
        .workspace
        .model()
        .dump()
        .iter()
        .map(|s| s.pseudo())
        .collect();
    assert_eq!(a, b);
}

#[test]
fn the_compiled_batch_replays_into_the_same_model() {
    let compiled = compile_fixture();
    let mut ws = Workspace::with_preset(&Preset::default_ontology()).unwrap();
    ws.execute(&compiled.batch).expect("the batch replays");
    let a: Vec<String> = compiled
        .workspace
        .model()
        .dump()
        .iter()
        .map(|s| s.pseudo())
        .collect();
    let b: Vec<String> = ws.model().dump().iter().map(|s| s.pseudo()).collect();
    assert_eq!(a, b);
    // And the dump replays over itself as pure noops.
    let dump = ws.model().dump();
    let outcomes = ws.execute(&dump).expect("dump replays");
    assert!(outcomes.iter().all(|o| matches!(o, Outcome::Noop)));
}
