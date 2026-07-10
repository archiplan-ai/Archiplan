//! The incidence core (`archi/requirements/scoring/the-matrix-joins-stress-to-structure.md`): pressure-surface
//! expansion, the S×N matrix, and the typed findings with their gates.

mod common;

use common::*;
use modeling_lang::{
    IncidenceConfig, IncidenceKind, IncidenceReport, IncidenceRow, Invariant, Model, Preset,
    Severity, StressOutcome, Workspace,
};
use serde_json::json;

fn ontology_ws() -> Workspace {
    Workspace::with_preset(&Preset::default_ontology()).expect("the default ontology loads")
}

fn row(id: &str, terms: &[&str], outcome: StressOutcome) -> IncidenceRow {
    IncidenceRow {
        id: id.to_string(),
        terms: terms.iter().map(|t| t.to_string()).collect(),
        outcome,
    }
}

fn incidence(model: &Model, rows: &[IncidenceRow], invariants: &[Invariant]) -> IncidenceReport {
    model.incidence(rows, invariants, &IncidenceConfig::default())
}

fn kinds(report: &IncidenceReport) -> Vec<&'static str> {
    report.findings.iter().map(|f| f.kind.name()).collect()
}

#[test]
fn a_type_expands_to_the_terms_it_transitively_classifies() {
    let mut ws = ontology_ws();
    // `Api` is itself a type (it classifies `Gateway`), so `Service`'s
    // surface holds its transitive *terms* — never the intermediate type.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Payments" },
            { "stmt": "define", "node": "Api" },
            { "stmt": "define", "node": "Gateway" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Payments" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Api" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Api", "target": "Gateway" }
        ]),
    );
    let model = ws.model();
    assert_eq!(
        model.term_surface("Service").unwrap(),
        ["Payments", "Gateway"]
    );
    assert_eq!(model.term_surface("Payments").unwrap(), ["Payments"]);
    assert_eq!(model.term_surface("Ghost"), None);

    // The matrix joins expanded rows against the frame's terms: `Api` is a
    // type, never a column.
    let rows = [
        row("wide", &["Payments", "Gateway"], StressOutcome::Surviving),
        row("narrow", &["Payments"], StressOutcome::Pending),
    ];
    let r = incidence(model, &rows, &[]);
    assert_eq!(r.matrix.components, ["Payments", "Gateway"]);
    assert_eq!(r.matrix.rows, [[1, 1], [1, 0]]);
    assert_eq!(r.scope.k_hyper, 0.75);
}

#[test]
fn response_similarity_splits_on_declared_connectivity() {
    let mut ws = ontology_ws();
    // A—B are directly connected; C and D touch only through `Mid`.
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "A" },
            { "stmt": "define", "node": "B" },
            { "stmt": "define", "node": "C" },
            { "stmt": "define", "node": "D" },
            { "stmt": "define", "node": "Mid" },
            { "stmt": "define", "rel": "calls", "directed": true, "source": "*", "target": "*" },
            { "stmt": "rel-edge", "rel": "calls", "source": "A", "target": "B" },
            { "stmt": "rel-edge", "rel": "calls", "source": "C", "target": "Mid" },
            { "stmt": "rel-edge", "rel": "calls", "source": "Mid", "target": "D" }
        ]),
    );
    let rows = [
        row("s1", &["A", "B", "C", "D"], StressOutcome::Surviving),
        row("s2", &["A", "B", "C", "D"], StressOutcome::Breaking),
        row("s3", &["A", "B"], StressOutcome::Pending),
    ];
    // Default depth 2: C reaches D through Mid, so both same-response pairs
    // read as merge candidates. J(A,C) = 2/3 stays under τ_J.
    let r = incidence(ws.model(), &rows, &[]);
    let merges: Vec<&str> = r
        .findings
        .iter()
        .filter_map(|f| match &f.kind {
            IncidenceKind::MergeCandidate { a, b, .. } => Some([a.as_str(), b.as_str()]),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(merges, ["A", "B", "C", "D"]);
    assert!(!kinds(&r).contains(&"hyperliminal_coupling"));

    // Depth 1: only the declared A—B edge connects; C—D becomes the hidden
    // coupling the matrix exists to surface.
    let config = IncidenceConfig {
        depth: 1,
        ..IncidenceConfig::default()
    };
    let r = ws.model().incidence(&rows, &[], &config);
    let hyper: Vec<(&str, usize)> = r
        .findings
        .iter()
        .filter_map(|f| match &f.kind {
            IncidenceKind::HyperliminalCoupling { a, shared, .. } => {
                Some((a.as_str(), shared.len()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(hyper, [("C", 2)]);
    assert!(kinds(&r).contains(&"merge_candidate"));

    // τ_J = 1.0 with a diverging third row: J(C,D) drops to 2/3 and the
    // pair falls out entirely.
    let rows = [
        row("s1", &["C", "D"], StressOutcome::Surviving),
        row("s2", &["C", "D"], StressOutcome::Surviving),
        row("s3", &["C"], StressOutcome::Surviving),
    ];
    let config = IncidenceConfig {
        tau_j: 1.0,
        ..IncidenceConfig::default()
    };
    let r = ws.model().incidence(&rows, &[], &config);
    assert!(
        !kinds(&r).contains(&"merge_candidate") && !kinds(&r).contains(&"hyperliminal_coupling"),
        "{:?}",
        kinds(&r)
    );
}

#[test]
fn compound_vulnerabilities_take_two_surviving_partial_covers() {
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "A" },
            { "stmt": "define", "node": "B" },
            { "stmt": "define", "node": "C" },
            { "stmt": "define", "node": "T" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "T", "target": "A" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "T", "target": "B" }
        ]),
    );
    let rows = [
        row("left", &["A"], StressOutcome::Surviving),
        row("right", &["B"], StressOutcome::Surviving),
        row("whole", &["A", "B"], StressOutcome::Surviving),
        row("broke", &["B"], StressOutcome::Breaking),
        row("undecided", &["B"], StressOutcome::Pending),
    ];
    // Two invariants naming the same terms — one by term paths, one through
    // the type `T` — and one naming nothing the frame knows.
    let invariants = [
        Invariant {
            id: "by-terms".to_string(),
            elements: vec!["A".to_string(), "B".to_string()],
        },
        Invariant {
            id: "by-type".to_string(),
            elements: vec!["T".to_string()],
        },
        Invariant {
            id: "ghostly".to_string(),
            elements: vec!["Ghost".to_string()],
        },
    ];
    let r = incidence(ws.model(), &rows, &invariants);
    let compounds: Vec<(&str, &str, &str)> = r
        .findings
        .iter()
        .filter_map(|f| match &f.kind {
            IncidenceKind::CompoundVulnerability {
                stressors,
                invariant,
                ..
            } => Some((stressors[0].as_str(), stressors[1].as_str(), invariant.as_str())),
            _ => None,
        })
        .collect();
    // Only `left` + `right` compound: `whole` covers alone (the architecture
    // already answered it), `broke` derived its requirements, `undecided`
    // has no verdict. Both resolving invariants fire; the ghost is skipped.
    assert_eq!(
        compounds,
        [("left", "right", "by-terms"), ("left", "right", "by-type")]
    );
    // Alerts sort first.
    assert_eq!(r.findings[0].severity, Severity::Alert);
    assert!(r.warnings.iter().all(|w| w.code != "NO_INVARIANTS"));
}

#[test]
fn hotspots_zero_columns_and_the_degenerate_warnings() {
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Hot" },
            { "stmt": "define", "node": "Warm" },
            { "stmt": "define", "node": "Cold" }
        ]),
    );
    let model = ws.model();
    let rows = [
        row("s1", &["Hot"], StressOutcome::Surviving),
        row("s2", &["Hot"], StressOutcome::Surviving),
        row("s3", &["Hot", "Warm"], StressOutcome::Surviving),
        row("s4", &["Gone", "Warm"], StressOutcome::Surviving),
    ];
    let r = incidence(model, &rows, &[]);
    // `Hot` at density 3/4 and `Warm` at exactly τ_D = 0.5 both report;
    // `Cold` is the zero column.
    let hotspots: Vec<(&str, usize)> = r
        .findings
        .iter()
        .filter_map(|f| match &f.kind {
            IncidenceKind::StressHotspot { node, hits, .. } => Some((node.as_str(), *hits)),
            _ => None,
        })
        .collect();
    assert_eq!(hotspots, [("Hot", 3), ("Warm", 2)]);
    let under: Vec<&str> = r
        .findings
        .iter()
        .filter_map(|f| match &f.kind {
            IncidenceKind::UnderStressed { node } => Some(node.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(under, ["Cold"]);
    // Findings sort severest-first: hotspots (warn) precede the zero
    // column (info).
    assert!(kinds(&r).ends_with(&["under_stressed"]));
    // The path the frame does not know is dropped, visibly.
    assert_eq!(r.scope.dropped["s4"], ["Gone"]);
    assert_eq!(r.matrix.rows[3], [0, 1, 0]);
    assert!(r.warnings.iter().any(|w| w.code == "DROPPED_AFFECTS"));

    // A single hit is never a hotspot, whatever its density.
    let r = incidence(model, &[row("only", &["Hot"], StressOutcome::Pending)], &[]);
    assert_eq!(kinds(&r), ["under_stressed", "under_stressed"]);

    // No stressors: an empty matrix warns instead of reporting every
    // column under-stressed.
    let r = incidence(model, &[], &[]);
    assert_eq!(kinds(&r), Vec::<&str>::new());
    assert!(r.warnings.iter().any(|w| w.code == "NO_STRESSORS"));
    assert_eq!(r.scope.k_hyper, 0.0);
}

/// The under-stressed sweep names behavior
/// (`archi/requirements/self-hosting/under-stressed-names-behavior.md`):
/// zero columns in `Data`'s closure are muted by default, `all_terms`
/// widens the sweep, and nothing else consults the filter.
#[test]
fn under_stressed_names_behavior_and_all_terms_widens() {
    let mut ws = ontology_ws();
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "Handler" },
            { "stmt": "define", "node": "Quiet" },
            { "stmt": "define", "node": "Journal" },
            { "stmt": "define", "node": "Tokens" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Service", "target": "Handler" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Data", "target": "Journal" },
            { "stmt": "rel-edge", "rel": "type_of", "source": "Data", "target": "Tokens" }
        ]),
    );
    let model = ws.model();
    let rows = [
        row("s1", &["Handler", "Journal"], StressOutcome::Surviving),
        row("s2", &["Journal"], StressOutcome::Breaking),
    ];
    let under = |r: &IncidenceReport| -> Vec<String> {
        r.findings
            .iter()
            .filter_map(|f| match &f.kind {
                IncidenceKind::UnderStressed { node } => Some(node.clone()),
                _ => None,
            })
            .collect()
    };

    // Default: the unpressed behavioral term stays loud; the unpressed data
    // term is muted. The pressed data term counts everywhere — full matrix,
    // hotspot findings — because only the emission consults the filter.
    let r = incidence(model, &rows, &[]);
    assert_eq!(r.matrix.components, ["Handler", "Quiet", "Journal", "Tokens"]);
    assert_eq!(under(&r), ["Quiet"]);
    assert!(
        r.findings.iter().any(|f| matches!(
            &f.kind,
            IncidenceKind::StressHotspot { node, hits, .. } if node == "Journal" && *hits == 2
        )),
        "{:?}",
        kinds(&r)
    );

    // all_terms widens the sweep back; matrix and the other findings are
    // identical either way.
    let all = model.incidence(
        &rows,
        &[],
        &IncidenceConfig {
            all_terms: true,
            ..IncidenceConfig::default()
        },
    );
    assert_eq!(under(&all), ["Quiet", "Tokens"]);
    assert_eq!(r.matrix.rows, all.matrix.rows);
    assert_eq!(r.matrix.components, all.matrix.components);
    let non_under = |r: &IncidenceReport| -> Vec<&'static str> {
        r.findings
            .iter()
            .filter(|f| !matches!(f.kind, IncidenceKind::UnderStressed { .. }))
            .map(|f| f.kind.name())
            .collect()
    };
    assert_eq!(non_under(&r), non_under(&all));

    // No `Data` in the preset: the closure is empty and the sweep is
    // complete — an unclassified term is never muted, even one that merely
    // borrows the name `Data` without classifying anything.
    let mut ws = Workspace::with_preset(&Preset::core()).expect("the core preset loads");
    outcomes(
        &mut ws,
        json!([
            { "stmt": "define", "node": "A" },
            { "stmt": "define", "node": "B" },
            { "stmt": "define", "node": "Data" }
        ]),
    );
    let r = incidence(
        ws.model(),
        &[row("s1", &["A"], StressOutcome::Surviving)],
        &[],
    );
    assert_eq!(under(&r), ["B", "Data"]);
}
