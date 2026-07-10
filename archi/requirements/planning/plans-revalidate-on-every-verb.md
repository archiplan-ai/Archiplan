---
kind: non-functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Plans revalidate on every verb

The editing surface splits like the rest of the system: authored fields — envelope prose,
task descriptions, `stack_details`, `inputs`, `outputs`, extra `spec_refs`,
`verifications`, scenarios — are edited directly in `plan.json`, exactly as requirements
are edited in their markdown; lifecycle state and derived content move only through
verbs — use, repin, task add, start, next, close, reset. Every verb re-reads and
re-validates the file on load, so a hand edit cannot drift silently: the next verb either
accepts the state whole or names what broke. Plans persist beside the other stores —
`archi/plans/<name>/plan.json`, wave indexes under `waves/`, and the `.current` marker
choosing the plan subsequent commands default to.

## System Context

One file, two writers — the author's editor and the tool's lifecycle — is safe only if
the tool never trusts its own last write. The same split governs sessions and the journal
(`rounds-fold-deliberately`, `link-truth-is-append-only`): text where authorship lives,
verbs where invariants do.

## Satisfy

`Planner` (loads, validates and re-derives on every verb; refuses latch and state
combinations the lifecycle forbids).

- test — plans::verify_flags_structure_and_notes_drift
- test — plans::the_lifecycle_captures_gates_and_latches_scenarios
