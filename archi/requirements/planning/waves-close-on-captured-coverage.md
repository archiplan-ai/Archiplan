---
kind: functional
origin: intent
satisfied-by: [Planner, Links]
deferred:
---

# Waves close on captured coverage

Opening a wave records the tree state its deltas are diffed against — a canonical
item-hash index, file to symbol to body hash, symbol-granular and git-free, so squashes
and shallow clones cannot break attribution. `plan next` closes the wave in one motion:
capture first — each closing task's delta minted into candidate links the closing agent
reviews (`capture-at-the-join`) — then two gates, structural verify and asserted coverage
of the spec_refs the delta pressed (`gates-press-the-delta` owns the pressed subset and
the suggestion checklist). `plan next` is re-runnable: a blocked gate is reviewed,
confirmed and retried, never forced. `plan current-wave` prints the tasks in flight;
`plan close` and `plan reset` are the manual overrides.

## System Context

The step that demands links must be the step that produces them — gate and capture firing
apart would let coverage debts pile up invisibly between waves. The wave index is also
what scopes multi-repo scans: `scans-see-every-mapped-member` records the scan set at
open.

## Satisfy

`Planner` (records the index at open, fires capture at close, gates on the folded
asserted set). `Links` (mints the wave delta into candidates and serves the fold the gate
reads).

- test — plan_e2e::the_plan_loop_produces_the_links_its_gate_demands
- test — capture::the_index_is_symbol_granular_and_formatting_blind
- test — plans::the_gate_presses_the_delta_and_suggests_the_rest
