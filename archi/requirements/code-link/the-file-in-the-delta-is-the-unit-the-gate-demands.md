---
kind: functional
origin: intent
satisfied-by: [Planner, Links.Capture]
deferred:
---

# The file in the delta is the unit the gate demands

`plan next` takes the files the wave's delta touched and demands that the declarations of
the tasks in flight, read as one set, name every one of them. A file no declaration names
holds the wave open, and the refusal names the file. It names no task: the tasks of a wave
share one tree, so nothing in the delta says who touched what. An entry may name any file
its writer touched, inside its task's `## Outputs` or not. Nothing else gates: no spec ref
is demanded, and no term is compared.

## System Context

The gate asked for spec refs, and it chose them by comparing the words of a changed file
against the words of a model name. It was wrong in both directions. It demanded `WorldDoc`
of a change to `world_check.rs` on the shared word `world`, and on the unit that replaced
capture it drew nothing for the two functions carrying the whole change while drawing three
for the helper that formats one diagnostic string.

The delta is the honest list, because git writes it and no actor in the wave can shrink it.
The file is the unit rather than the symbol, because the symbol was tried: it asked thirty
entries for two changed files, and twenty-seven of them were tests, fixtures and string
constants that answer no part of a model (`a-gate-cheap-enough-to-be-paid`). A file is one
entry when one entry is true, and several when the file serves several nodes.

What the gate stops proving is the spec side. A model element that no code answers is the
audit's `unlinked_spec_ref` finding, which reports it and never blocks — advising on a fact
instead of blocking on a resemblance.

## Satisfy

`Planner` (the wave gate demands the delta's files against the union of the declarations,
and its refusal names the file). `Links.Capture` (the delta is the file list; no term
comparison, no per-task claim map, no suppressed count).

- test — a wave whose delta holds a file no declaration names refuses, and the refusal names that file
- test — the refusal names the file and no task, with two tasks in flight
- test — a declaration naming a file outside its task's `## Outputs` satisfies the gate for that file
- test — one file named by two entries against two different elements satisfies the gate once
- test — a wave whose declarations name every file in the delta closes with no spec ref demanded
- test — `plan next` prints no `no-signal pair` count
