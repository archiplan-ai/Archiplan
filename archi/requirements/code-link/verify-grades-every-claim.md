---
kind: functional
origin: intent
satisfied-by: [Links.Grader]
deferred:
---

# Verify grades every claim

`link verify` recomputes every projection in scope and grades it. Clean: the anchor
resolves and the watched hash matches. Drifted: the anchor resolves but the watched hash
moved — review whether spec or code is authoritative, then re-pin or fix. Moved: the
anchor is gone and a heuristic candidate exists elsewhere — `repin --to` rewrites the
projection while the birth record stays untouched; evidence auto-follows at reduced
confidence. Missing: nothing resolves — broken for an asserted link, restore the code or
retire; decay for evidence. CanonicalizerMismatch: the ruler changed — rehash before
trusting any comparison (`hash-contract-is-versioned`). Unreachable: the member's
checkout is absent — upstream of Missing and never a decay (`absence-is-not-drift`). CI
reads the grades as exit codes: Missing and CanonicalizerMismatch fail, Drifted fails
only on asserted literal links, evidence states never fail. Spec-side drift mirrors it: a
SpecRef resolves at its pin by construction but may not at Working — the rename that
orphaned it is locatable in the version chain, so migration is mechanical or the link
retires with its element.

## System Context

The grading table is the tool's whole answer to "does the graph still match the tree";
every state must map to one operator move, or the alarm trains skimming —
`drift-graded-per-kind` picks the watched hash per link kind.

## Satisfy

`Links.Grader` (recomputes projections, grades the states, maps them onto verify's exit
codes).

- test — links::add_verify_and_the_drift_grades
- test — links::moves_are_candidates_and_deletions_are_missing
- test — links::evidence_confirms_decays_and_prunes
