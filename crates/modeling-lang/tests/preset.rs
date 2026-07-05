//! Presets loaded as the stdlib: visibility, protection and round-trips
//! (`requirements/modeling-lang/ontology.md`).

mod common;

use common::*;
use modeling_lang::{ErrorCode, Layer, Preset, Statement, Workspace};
use serde_json::json;

fn ontology_ws() -> Workspace {
    Workspace::with_preset(&Preset::default_ontology()).expect("the default ontology loads")
}

#[test]
fn default_ontology_loads_sealed_and_queryable() {
    let mut ws = ontology_ws();
    assert_eq!(ws.revision(), 0, "a preset load is not a model change");
    assert_eq!(ws.model().preset_name(), "default");
    // Ontology nodes are queryable like any node...
    let ids = node_ids(&mut ws, json!({ "stmt": "query" }));
    assert_eq!(ids, ["Data", "Service", "Function", "Storage"]);
    // ...but the dump omits them: the stdlib is substrate, and a restore
    // loads the same preset first.
    assert_eq!(dump_pseudo(&ws), Vec::<String>::new());
    // The uninstantiated ontology is not reported by check.
    assert_eq!(findings(&mut ws, json!({ "stmt": "check" })), vec![]);
}

#[test]
fn preset_elements_are_protected() {
    let mut ws = ontology_ws();
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "delete", "node": "Data" })),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "rename", "node": "Service", "to": "Svc" })
        ),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "delete", "rel": "type_of" })),
        ErrorCode::StdlibProtected
    );
    // Identical restatement of a preset element is still a safe no-op.
    assert!(is_noop(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "Data" })
    )));
    // Users may augment a preset node's scope and manage their own additions...
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "define", "node": "Data.Schema" })
    )));
    // ...but redefining the preset node (which would cascade the
    // augmentation away) is protected, while an addressed delete is not.
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "redefine", "node": "Data" })),
        ErrorCode::StdlibProtected
    );
    assert!(is_applied(&outcome(
        &mut ws,
        json!({ "stmt": "delete", "node": "Data.Schema" })
    )));
}

#[test]
fn preset_edges_and_types_are_protected() {
    // A preset with its own wiring: a sort taxonomy over the ontology.
    let preset = Preset::from_value(
        "sorted",
        &json!([
            { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
              "source": "*", "target": "*" },
            { "stmt": "define", "rel": "of_sort", "trans": true, "directed": true,
              "source": "*", "target": "*" },
            { "stmt": "define", "conn": "pipe", "directed": true, "source": "*", "target": "*" },
            { "stmt": "define", "node": "Functional" },
            { "stmt": "define", "node": "Service" },
            { "stmt": "rel-edge", "rel": "of_sort", "source": "Service", "target": "Functional" }
        ]),
    )
    .expect("the preset parses");
    let mut ws = Workspace::with_preset(&preset).expect("the preset loads");
    outcomes(&mut ws, json!([{ "stmt": "define", "view": "v" }]));

    let edge = json!({ "stmt": "rel-edge", "rel": "of_sort",
                       "source": "Service", "target": "Functional" });
    // Identical restatement: noop. Tagging into a view, untagging, deleting:
    // protected — tags on a stdlib edge would not survive a dump replay.
    assert!(is_noop(&outcome(&mut ws, edge.clone())));
    let mut tagged = edge.clone();
    tagged["views"] = json!(["v"]);
    assert_eq!(err_code(&mut ws, tagged), ErrorCode::StdlibProtected);
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "untag", "edge": edge, "views": ["v"] })
        ),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "delete", "edge": { "stmt": "rel-edge", "rel": "of_sort",
                                                "source": "Service", "target": "Functional" } })
        ),
        ErrorCode::StdlibProtected
    );
    // Preset rel/conn types cannot be redefined divergently or deleted.
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "redefine", "rel": "of_sort", "directed": false,
                    "source": "*", "target": "*" })
        ),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(
            &mut ws,
            json!({ "stmt": "redefine", "conn": "pipe", "directed": false,
                    "source": "*", "target": "*" })
        ),
        ErrorCode::StdlibProtected
    );
    assert_eq!(
        err_code(&mut ws, json!({ "stmt": "delete", "conn": "pipe" })),
        ErrorCode::StdlibProtected
    );
}

#[test]
fn preset_validation() {
    // Presets hold creation statements only.
    let e = Preset::from_value("bad", &json!([{ "stmt": "check" }])).unwrap_err();
    assert_eq!(e.code, ErrorCode::PresetInvalid);
    // The classifier `type_of` must exist...
    let empty = Preset::from_value("empty", &json!([])).expect("parses");
    let e = Workspace::with_preset(&empty).unwrap_err();
    assert_eq!(e.code, ErrorCode::PresetInvalid);
    // ...and conform to `rel trans type_of := * -> *`.
    let flat = Preset::from_value(
        "flat",
        &json!([
            { "stmt": "define", "rel": "type_of", "directed": true, "source": "*", "target": "*" }
        ]),
    )
    .expect("parses");
    let e = Workspace::with_preset(&flat).unwrap_err();
    assert_eq!(e.code, ErrorCode::PresetInvalid);
    // A rejected preset statement fails the load, reported as the preset's.
    let broken = Preset::from_value(
        "broken",
        &json!([
            { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
              "source": "*", "target": "*" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Ghost", "target": "Ghost" }
        ]),
    )
    .expect("parses");
    let e = Workspace::with_preset(&broken).unwrap_err();
    assert_eq!(e.code, ErrorCode::PresetInvalid);
}

#[test]
fn user_content_round_trips_on_a_preset() {
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Payments" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" }
        ]),
    );
    // Layers key off the preset's classifier.
    assert_eq!(ws.model().layer_of("Service"), Some(Layer::Epistemic));
    assert_eq!(ws.model().layer_of("Payments"), Some(Layer::Epistatic));

    // The dump holds user statements only, referencing preset nodes by path.
    let dumped = ws.model().dump();
    let lines: Vec<String> = dumped.iter().map(Statement::pseudo).collect();
    assert_eq!(lines, ["def node Payments;", "Service type_of Payments;"]);

    // Restoring with the same preset replays identically.
    let replayed = Workspace::restore(&Preset::default_ontology(), ws.revision(), &dumped)
        .expect("a dump replays on its preset");
    assert_eq!(replayed.model().dump(), dumped);

    // Restoring on the wrong preset fails loudly: the dump names `Service`.
    let e = Workspace::restore(&Preset::core(), ws.revision(), &dumped).unwrap_err();
    assert_eq!(e.code, ErrorCode::UnknownName);
}
