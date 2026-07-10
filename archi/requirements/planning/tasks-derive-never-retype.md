---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Tasks derive, never retype

One task per node: `plan task add <node>` pins the task to one node in one scope and
seeds its `spec_refs` with the node plus its incoming edges — the contracts not to break.
Requirements are pulled, never typed: for every spec_ref, the requirements whose
`satisfied-by` expansion against the pinned version intersects it — edge refs matching
through their endpoint nodes — arrive as matches, recomputed on every read, because
requirements are living documents a stored match set could only misrepresent. Each match
carries `matched_refs` (which of the task's refs pulled it in) and a derived `slot_id`
ordinal for short addresses; identity stays the slug. Dependencies flow through `inputs`,
each keyed by the producing task — the single source of inter-task order — and `outputs`
declares the files the task will write, the boundary capture later attributes its delta
through.

## System Context

Execution left to itself grows a shadow copy of the spec — task lists that re-type
requirements by hand, correct the day they are written. Deriving from the pin removes the
transcription class of drift, and `capture-at-the-join` depends on spec_refs existing
before the first line of code does.

## Satisfy

`Planner` (seeds refs from the pinned graph, recomputes the reverse lookup per verb, and
validates inputs and outputs structurally).

- test — plans::task_add_seeds_the_node_and_its_incoming_edges
- test — plans::the_reverse_lookup_matches_terms_types_and_edge_endpoints
