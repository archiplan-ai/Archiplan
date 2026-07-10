---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# A plan projects a pinned spec

`plan use <name>` switches to a named plan, creating the skeleton on first use, and pins
the version the live model is at — refusing a dirty or unversioned model, because a plan
projects a hardened spec and hardening is `version save`. The envelope frames the work
before any task is cut: the `problem`, the `technology_stack` with provenance, an
`architecture_summary` of one-line roles per top-level node, and the `stack_mapping` from
concrete tech to summary nodes. When the spec advances mid-plan, `plan verify` notes it;
`plan repin` re-pins the active plan to the version the live model is at, under the same
refusals, and the next verify flags every task whose obligations no longer hold.

## System Context

The spec says what must be true; the plan says how the work is sliced into tasks and
waves. An unpinned plan would inherit every mid-flight model edit as silent obligation
drift — the exact disease projection exists to end.

## Satisfy

`Planner` (pins on use and repin, refuses unhardened models, validates the envelope, and
re-derives obligations against the pin).

- test — plans::use_pins_a_hardened_version_and_switches
- test — plans::verify_flags_structure_and_notes_drift
