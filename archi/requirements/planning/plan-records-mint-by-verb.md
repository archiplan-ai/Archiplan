---
kind: functional
origin: intent
satisfied-by: [Planner, Cli]
deferred:
---

# Plan records mint by verb

Plan records follow the doc boundary: creation and removal are verbs, prose is
edited in place. `plan use` mints the folder — charter, `scenarios.md`,
`state.json` with the pinned version; `plan task add <node>` mints a task file
with its spec refs seeded from the model; `plan task rm <id>` retires one after
a pre-flight — a task another task's inputs name, or a plan past draft, refuses
with the list. Re-minting an identical skeleton converges; everything else —
ownership, inputs, outputs, verifications, scenarios — is edited in the files
and `plan verify` holds the worklist.

## System Context

The prose verbs were argv transport: every field of the old plan flowed through
CLI arguments because hand edits were forbidden on JSON. With records as
markdown the editor is the pen and the verbs shrink to what only they can do —
derive spec refs from the model, pre-flight a removal, move lifecycle. The
write-time candidate check on ownership moves with it: `owns` is hand-curated
frontmatter now, and verify — already the gate the start stands on — holds
owns ⊆ matched. Removal of a whole plan folder stays deferred: its blast
radius (seat bindings, captured waves) wants its own design.

## Satisfy

`Planner` (the mint writes seeded skeletons, the removal pre-flights the DAG
and the state); `Cli` (the trimmed verb surface: mint, rm and lifecycle beside
the untouched reads).

- test — task add seeds spec refs from the node and re-mints converge; a foreign edit refuses
- test — task rm refuses while another task's inputs name it, and outside draft
- test — a hand-typed owns entry outside the matched set fails verify, gating the start
