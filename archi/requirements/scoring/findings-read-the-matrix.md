---
kind: functional
origin: intent
satisfied-by: [Incidence]
deferred:
---

# Findings read the matrix

The matrix's findings are typed, severity-tagged (info < warn < alert) and append-only.
`compound_vulnerability` (alert): two surviving stressors, neither alone covering an
invariant — the satisfaction claim of an intent-origin requirement, expanded against the
frame — whose union of affected terms does: individually answered, jointly a broken
initial promise. `hyperliminal_coupling` (warn): two columns with near-identical stressor
sets — Jaccard at least τ_J over two or more shared stressors — and no declared path
between them: a hidden dependency the matrix made visible. `stress_hotspot` (warn): a
column pressed by two or more stressors carrying a τ_D share of the scope.
`density_alert` (alert): the matrix denser than τ_K over three or more stressors — stress
is landing everywhere at once. `boundary_crossing_stressor` (warn, alert past w̄ + 2σ): a
row pressing two or more terms and more of the frame than typical — w > w̄ + σ over the
scope's row weights — likely crossing a boundary the architecture should make explicit.
`merge_candidate` (info): the same similarity over a declared path — two nodes that might
really be one, or a shared concern worth extracting. `under_stressed` (info): a zero
column — `under-stressed-names-behavior` scopes its default to behavioral terms. A
declared path walks what the model actually declares — relation edges except the
classifier, connections between the port-owning nodes, applications, containment —
bounded by `--depth` and `--path-limit`; an exhausted budget assumes the pair connected,
suppressing a finding rather than fabricating one, and warns `PATH_LIMIT_HIT`. Reports
ship human by default and JSON by flag (`--json`, `--matrix`, `--k-hyper`, `--findings`,
`--no-matrix`; `--kind` and `--min-severity` filter every mode), and the report
auto-fires on the save that closes a session — its failure a warning, never a failed
save, `ARCHI_REPORT_JSON=1` switching the auto-report to JSON.

## System Context

Breaking stressors already bent the architecture and derived their requirements; the
compound finding exists for promises that break only in combination. One similarity
signal reads two ways — corroboration over a declared path, contradiction over none — and
the split is exactly the declared-edge traversal.

## Satisfy

`Incidence` (the finding emitters over the matrix, the declared-path walk with its
budgets, the severity and kind filters, the auto-fire hook on the closing save).

- test — incidence::response_similarity_splits_on_declared_connectivity
- test — incidence::compound_vulnerabilities_take_two_surviving_partial_covers
- test — incidence::hotspots_zero_columns_and_the_degenerate_warnings
- test — incidence::density_and_boundary_crossing_read_scope_wide_pressure
- test — incidence::invariants_are_the_intent_origin_claims
