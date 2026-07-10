//! NKP slicing and metrics (`archi/requirements/scoring/the-landscape-is-a-slice.md`): the exclusion
//! patterns, scope modes, delegation folding, and the landscape numbers.

mod common;

use common::*;
use modeling_lang::{
    CorridorAction, CorridorLabel, ExcludePattern, Neutrality, NkpConfig, NkpReport, NkpScope,
    Preset, Regime, Workspace,
};
use serde_json::json;

fn ontology_ws() -> Workspace {
    Workspace::with_preset(&Preset::default_ontology()).expect("the default ontology loads")
}

/// Three services calling each other, a data node written by one of them and
/// carried on a connection between two of them.
fn services_ws() -> Workspace {
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Payments" },
            { "stmt": "define", "node": "Orders" },
            { "stmt": "define", "node": "Shipping" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Orders" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Shipping" },
            { "stmt": "define", "node": "OrderId" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Data", "target": "OrderId" },
            { "stmt": "define", "rel": "calls", "directed": true,
              "source": { "anchor": "Service", "rel": "type_of" },
              "target": { "anchor": "Service", "rel": "type_of" } },
            { "stmt": "define", "rel": "writes", "directed": true, "source": "*", "target": "*" },
            { "stmt": "rel-edge", "rel": "calls", "source": "Payments", "target": "Orders" },
            { "stmt": "rel-edge", "rel": "calls", "source": "Orders", "target": "Shipping" },
            { "stmt": "rel-edge", "rel": "calls", "source": "Payments", "target": "Shipping" },
            { "stmt": "rel-edge", "rel": "writes", "source": "Payments", "target": "OrderId" },
            { "stmt": "define", "conn": "send", "directed": true,
              "source":  { "anchor": "Service", "rel": "type_of" },
              "carrier": { "anchor": "Data", "rel": "type_of" },
              "target":  { "anchor": "Service", "rel": "type_of" } },
            { "stmt": "conn-edge", "conn": "send",
              "source": { "node": "Payments", "port": "out" }, "carrier": "OrderId",
              "target": { "node": "Orders", "port": "in" } }
        ]),
    );
    ws
}

fn nkp(ws: &Workspace, config: &NkpConfig) -> NkpReport {
    ws.model().nkp(config).expect("the analysis runs")
}

#[test]
fn default_slice_keeps_behavior_drops_data_types_and_preset() {
    let r = nkp(&services_ws(), &NkpConfig::default());
    // Types (`_ type_of *`), Data instances (`Data type_of _`) and the
    // preset scaffolding are all out; the three services remain.
    assert_eq!(r.matrix.nodes, ["Payments", "Orders", "Shipping"]);
    assert_eq!(r.scope.excluded_nodes, 1, "OrderId is a Data instance");
    // The carrier-bearing connection still couples Payments → Orders; the
    // `writes` edge onto the data node is gone with its endpoint.
    assert_eq!(
        r.matrix.rows,
        vec![vec![0, 0, 0], vec![1, 0, 0], vec![1, 1, 0]]
    );
    assert_eq!(r.scope.edge_count, 3);
    assert_eq!(r.metrics.k_bar, 1.0);
    assert_eq!(r.metrics.regime, Regime::Critical);
    assert_eq!(r.metrics.correlation_length, Some(1.0));
    // Shipping takes two distinct sources; K̄ + σ ≈ 1.82.
    assert_eq!(r.hotspots.len(), 1);
    assert_eq!(r.hotspots[0].node, "Shipping");
    assert_eq!(r.hotspots[0].k_in, 2);
}

#[test]
fn only_edge_types_narrows_coupling() {
    let config = NkpConfig {
        only_edge_types: Some(vec!["send".to_string()]),
        ..NkpConfig::default()
    };
    let r = nkp(&services_ws(), &config);
    assert_eq!(
        r.matrix.rows,
        vec![vec![0, 0, 0], vec![1, 0, 0], vec![0, 0, 0]],
        "only the connection counts"
    );
    assert_eq!(r.metrics.regime, Regime::Ordered);
}

#[test]
fn exclusion_follows_the_transitive_closure() {
    let mut ws = services_ws();
    // Receipt is typed by RichData, itself typed by Data: `Data type_of _`
    // reaches it transitively. RichData is a type (`_ type_of *`) as well.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "RichData" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Data", "target": "RichData" },
            { "stmt": "define", "node": "Receipt" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "RichData", "target": "Receipt" }
        ]),
    );
    let r = nkp(&ws, &NkpConfig::default());
    assert_eq!(r.matrix.nodes, ["Payments", "Orders", "Shipping"]);
    assert_eq!(
        r.scope.excluded_nodes, 3,
        "OrderId, RichData and Receipt are all excluded"
    );
}

#[test]
fn custom_patterns_and_unknown_refs() {
    // Structural validation is hard: exactly one `_`.
    assert!(ExcludePattern::parse("_ type_of _").is_err());
    assert!(ExcludePattern::parse("* type_of *").is_err());
    assert!(ExcludePattern::parse("junk").is_err());

    let ws = services_ws();
    // Excluding all Service instances empties the landscape.
    let mut config = NkpConfig::default();
    config
        .exclude
        .push(ExcludePattern::parse("Service type_of _").expect("parses"));
    let r = nkp(&ws, &config);
    assert_eq!(r.scope.node_count, 0);
    assert_eq!(r.scope.excluded_nodes, 4);
    assert!(r.warnings.iter().any(|w| w.code == "SMALL_N"));

    // Unresolvable references match nothing and warn instead of failing.
    let mut config = NkpConfig::default();
    config
        .exclude
        .push(ExcludePattern::parse("_ bogus *").expect("parses"));
    config
        .exclude
        .push(ExcludePattern::parse("Ghost type_of _").expect("parses"));
    let r = nkp(&ws, &config);
    assert_eq!(r.scope.node_count, 3);
    assert_eq!(
        r.warnings
            .iter()
            .filter(|w| w.code == "UNKNOWN_EXCLUDE_REF")
            .count(),
        2
    );
}

#[test]
fn folding_and_scope_modes() {
    let mut ws = services_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Orders.Handler" },
            { "stmt": "app", "node": "Orders", "port": "in",
              "inner": { "node": "Handler", "port": "handle" } },
            { "stmt": "define", "node": "Orders.Helper" },
            { "stmt": "define", "rel": "uses", "directed": true, "source": "*", "target": "*" },
            { "stmt": "rel-edge", "rel": "uses",
              "source": "Orders.Handler", "target": "Orders.Helper" }
        ]),
    );

    // Recursive: the delegated Handler folds into Orders, and its `uses`
    // coupling re-attaches as Orders → Helper.
    let r = nkp(&ws, &NkpConfig::default());
    assert_eq!(
        r.matrix.nodes,
        ["Payments", "Orders", "Shipping", "Orders.Helper"]
    );
    assert_eq!(
        r.scope.folded.get("Orders"),
        Some(&vec!["Orders.Handler".to_string()])
    );
    assert_eq!(r.scope.edge_count, 4);

    // Top level: inner nodes are out of scope entirely.
    let top = nkp(
        &ws,
        &NkpConfig {
            scope: NkpScope::TopLevel,
            ..NkpConfig::default()
        },
    );
    assert_eq!(top.matrix.nodes, ["Payments", "Orders", "Shipping"]);

    // Children of Orders: one level, no folding — the delegated Handler *is*
    // a component of that area.
    let inner = nkp(
        &ws,
        &NkpConfig {
            scope: NkpScope::Children("Orders".to_string()),
            ..NkpConfig::default()
        },
    );
    assert_eq!(inner.matrix.nodes, ["Orders.Handler", "Orders.Helper"]);
    assert_eq!(inner.scope.edge_count, 1);

    // An unknown scope path is a hard error, not a warning.
    assert!(
        ws.model()
            .nkp(&NkpConfig {
                scope: NkpScope::Children("Nope".to_string()),
                ..NkpConfig::default()
            })
            .is_err()
    );
}

#[test]
fn hub_hotspot_and_corridors() {
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "A" },
            { "stmt": "define", "node": "B" },
            { "stmt": "define", "node": "C" },
            { "stmt": "define", "node": "D" },
            { "stmt": "define", "node": "Hub" },
            { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" },
            { "stmt": "rel-edge", "rel": "dep", "source": "A", "target": "Hub" },
            { "stmt": "rel-edge", "rel": "dep", "source": "B", "target": "Hub" },
            { "stmt": "rel-edge", "rel": "dep", "source": "C", "target": "Hub" },
            { "stmt": "rel-edge", "rel": "dep", "source": "D", "target": "Hub" }
        ]),
    );
    let r = nkp(&ws, &NkpConfig::default());
    // K = [0, 0, 0, 0, 4]: K̄ = 0.8 (ordered), σ = 1.6, so only Hub is hot.
    assert_eq!(r.metrics.regime, Regime::Ordered);
    assert_eq!(r.hotspots.len(), 1);
    assert_eq!(r.hotspots[0].node, "Hub");
    // Degree-derived P: the spokes are fully neutral, the hub is not. Each
    // spoke is its own corridor, fully exposed to the hub.
    assert_eq!(r.neutral_corridors.len(), 4);
    for c in &r.neutral_corridors {
        assert_eq!(c.label, CorridorLabel::PartiallyNeutral);
        assert_eq!(c.boundary_exposure, 1.0);
        assert_eq!(c.action, None);
        assert_eq!(c.confidence, 0.0);
    }
    // Uniform neutrality pulls the hub in: one safe corridor, encapsulable.
    let uni = nkp(
        &ws,
        &NkpConfig {
            neutrality: Neutrality::Uniform(0.9),
            ..NkpConfig::default()
        },
    );
    assert_eq!(uni.neutral_corridors.len(), 1);
    let c = &uni.neutral_corridors[0];
    assert_eq!(c.nodes.len(), 5);
    assert_eq!(c.label, CorridorLabel::Safe);
    assert_eq!(c.action, Some(CorridorAction::Encapsulate));
    assert_eq!(c.confidence, 1.0);
}

#[test]
fn degenerate_and_disconnected_graphs() {
    // Ontology only: nothing to analyze, reported rather than crashed.
    let r = nkp(&ontology_ws(), &NkpConfig::default());
    assert_eq!(r.scope.node_count, 0);
    assert_eq!(r.metrics.regime, Regime::Ordered);
    assert!(r.matrix.nodes.is_empty());
    assert!(r.neutral_corridors.is_empty());
    assert!(r.warnings.iter().any(|w| w.code == "SMALL_N"));

    // Two isolated pairs: disconnection is warned, metrics aggregate.
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "E" },
            { "stmt": "define", "node": "F" },
            { "stmt": "define", "node": "G" },
            { "stmt": "define", "node": "H" },
            { "stmt": "define", "rel": "dep", "directed": true, "source": "*", "target": "*" },
            { "stmt": "rel-edge", "rel": "dep", "source": "E", "target": "F" },
            { "stmt": "rel-edge", "rel": "dep", "source": "G", "target": "H" }
        ]),
    );
    let r = nkp(&ws, &NkpConfig::default());
    assert_eq!(r.metrics.k_bar, 0.5);
    assert!(r.warnings.iter().any(|w| w.code == "DISCONNECTED"));
}
