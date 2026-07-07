# Plan

A plan projects a hardened spec into an executable task graph. Each
task is pinned to one node in one scope; its requirements, spec
references, inputs, and outputs are derived from the spec rather than
invented.

## Scope

Scope specifies what the plan implements. It may target:
- A certain intent
- A set of requirements that are not implemented yet (don't have
  asserted code-links)

## Capabilities

- **Use / create a plan** — switch to a named plan, creating an empty
  skeleton on first use. The plan pins the current spec version.
- **Envelope** — author the plan's `problem`, `technology_stack`
  (with provenance), `architecture_summary` (one-line role per
  top-level node), and `stack_mapping` (which concrete tech realizes
  which summary node). These frame the plan before any task is cut.
- **Tasks** — one task per node. Each carries a description, the
  owning scope and node, `spec_refs` (the spec elements it realizes —
  auto-seeded on create as the node plus its incoming edges),
  `stack_details`, `inputs` (each keyed by the producing task — the
  inputs list is the single source of truth for inter-task
  dependencies), and `outputs` (files written).
- **Auto-derived requirements** — for every `spec_ref` the task
  lists, the requirements targeting it are pulled from the spec via
  reverse lookup and attached. Cross-scope edge refs also pull reqs
  on their non-excluded endpoints. Spec-side identity is
  `(req_id, home_scope)` — two scopes can mint reqs with the same id
  and they stay distinct. Each matched req carries `matched_refs`
  (which of the task's own spec_refs pulled it in) and a stable
  `slot_id` (`r1`, `r2`, …) — the local address CLI verbs like
  `verification add/remove` use to disambiguate colliding ids on the
  same task.
- **Verifications** — authored per task per requirement: how the
  implementer will prove the req is met (a failing test, a runtime
  contract, a migration assertion, anything the prose prescribes).
  Verifications live on the plan, not on the requirement itself.
- **Scenarios** — free-text user stories on the plan envelope,
  decoupled from spec requirements. They never become tasks. After
  the last task wave is closed, `archi plan next` prints the
  scenarios block as the final step (`scenarios_displayed` latch is
  set). One more `archi plan next` sets `scenarios_closed`, prints
  `DONE`, and transitions the plan to `Completed`. `plan reset`
  unlatches both flags so the cycle can run again. If no scenarios
  are recorded, `plan next` skips the step and closes the plan as
  before.
- **Lifecycle** — `plan verify` checks structural invariants against
  the current spec. `plan start` refuses to transition until every
  matched requirement has ≥1 verification and the plan is
  structurally clean. Opening a wave records the tree state each of
  its tasks' deltas is later diffed against. `plan next` first runs
  the code-link capture — each closing task's delta is minted into
  candidate links the closing agent reviews and selectively asserts
  ([code-link.md](code-link.md#code-links--tasks)) — then advances
  the wave under two gates: structural verify (same checks as
  `plan start`) and asserted code-link coverage of every active
  task's spec_refs at the Working version. The step that demands
  links is the step that produces them. `plan current-wave` prints
  the tasks in flight. `plan close` and `plan reset` are manual
  overrides.

## Why

The spec says *what* must be true; the plan says *how the work is
sliced into tasks and waves*. Keeping them apart lets the spec stay
about architecture and the plan stay about execution. Pulling
requirements and spec_refs from the spec rather than retyping them
removes a class of drift: when the spec changes, `plan verify` flags
every task whose obligations no longer hold.

Scenarios sit on the plan, not the spec, on purpose. A user story
crosses many requirements across many nodes; pinning it to a single
spec element would lie about its scope. Composing the story into an
end-to-end task in the final wave is execution-shaped, which is the
plan's job.

## Persistence

Each plan lives at `archi/plans/<name>/plan.json`, beside the
version archive and the link journal. The marker
`archi/plans/.current` records which plan subsequent commands
default to.

## Cross-references

- [`skills/archiplan.md`](../skills/archiplan.md) — the agent
  workflow that drives plan authoring end to end.
- [`code-link.md`](code-link.md) — capture at wave close, and how
  asserted links gate `plan next` wave transitions.
