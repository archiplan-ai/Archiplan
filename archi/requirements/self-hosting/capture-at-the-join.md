---
kind: functional
origin: intent
satisfied-by: [Planner, Links]
deferred:
---

# Capture at the join

Traceability is captured where intent is known — the task — not recovered after the fact. A task
carries its spec_refs before the first line of code exists; when its wave closes, the changed
symbols arrive pre-attributed as candidate links, the closing agent asserts the load-bearing
ones, and only asserted links count toward the gate that lets the next wave open.

## System Context

Wave deltas are read off a canonical item-hash index taken at wave open — symbol-granular and
git-free, so squashes and shallow clones cannot break attribution.

## Satisfy

`Planner` records the index when a wave opens and refuses to advance until every active task's
spec_refs carry asserted coverage; `Links` mints the wave's delta into evidence links that a
decision — confirm — raises to asserted. Evidence never gates.

- test — plan_e2e::the_plan_loop_produces_the_links_its_gate_demands drives the full loop end to end
