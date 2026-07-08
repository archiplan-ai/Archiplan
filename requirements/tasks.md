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
  reverse lookup: a requirement matches when its `satisfied-by`
  expansion ([requirements.md](requirements.md#satisfy), expanded
  against the pinned version) intersects the task's spec_refs; edge
  refs match through their endpoint nodes. Requirement identity is
  the slug — slugs are unique project-wide
  ([requirements.md](requirements.md#slugs), `E_SLUG`), so nothing
  disambiguates further. Matched reqs are recomputed, never stored:
  requirements are living documents outside the version archive, so
  a stored match set could only go stale. Each match carries
  `matched_refs` (which of the task's own spec_refs pulled it in)
  and a `slot_id` (`r1`, `r2`, …) — a derived per-task ordinal for
  short addresses in reports, not an identity.
- **Verifications** — authored per task per requirement: how the
  implementer will prove the req is met (a failing test, a runtime
  contract, a migration assertion, anything the prose prescribes).
  Verifications live on the plan, not on the requirement itself,
  keyed by the requirement's slug in the task's `verifications` map.
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
  its tasks' deltas is later diffed against — a canonical item-hash
  index (file → symbol → body hash, by the code-link canonicalizer),
  so deltas are symbol-granular and git-free by construction.
  `plan next` first runs the code-link capture — each closing task's
  delta is minted into candidate links the closing agent reviews and
  selectively asserts
  ([code-link.md](code-link.md#code-links--tasks)) — then advances
  the wave under two gates: structural verify (same checks as
  `plan start`) and asserted code-link coverage at the Working
  version of the spec_refs the closing delta **presses** — the refs
  some claimed changed item carries term signal for, by the same
  test capture mints with. Unpressed refs never block: the uncovered
  ones print as a suggested checklist of exact
  `archi link add <ref> <file#symbol> --kind indirect` lines — on
  the blocked message and on the passing close alike — so
  hand-authoring surface the delta did not touch is a named,
  voluntary move, and a delta pressing nothing closes its wave
  without demanding links. The step that demands links is the step
  that produces them; `plan next` is re-runnable, so a failed gate
  is reviewed (`link confirm`) and retried. `plan current-wave`
  prints the tasks in flight. `plan close` and `plan reset` are
  manual overrides.
- **Pinning** — `plan use` pins the version the live model is *at*
  and refuses on a dirty or unversioned model: a plan projects a
  hardened spec, and hardening is `archi version save`. When the
  spec advances mid-plan, `plan verify` notes it; `plan repin`
  re-pins the active plan to the version the live model is at (same
  refusals), and the next `plan verify` flags every task whose
  obligations no longer hold.

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
version archive and the link journal; wave-open indexes sit under
`archi/plans/<name>/waves/`. The marker `archi/plans/.current`
records which plan subsequent commands default to.

The editing surface splits like the rest of the system: authored
fields — envelope prose, task descriptions, `stack_details`,
`inputs`, `outputs`, extra `spec_refs`, `verifications`, scenarios —
are edited in `plan.json` directly, exactly as requirements are
edited in their markdown ([requirements.md](requirements.md));
lifecycle state and derived content move only through verbs
(`use`, `repin`, `task add`, `start`, `next`, `close`, `reset`), and
every verb re-validates the file on load, so a hand edit cannot
drift silently.

## Cross-references

- [`skills/archi.md`](../skills/archi.md) — the agent workflow that
  drives plan authoring end to end.
- [`code-link.md`](code-link.md) — capture at wave close, and how
  asserted links gate `plan next` wave transitions.
