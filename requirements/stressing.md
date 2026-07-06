# Stressing

A stress session is a batch of stressors applied against the current
version. Each stressor is a hypothesized failure mode, scale concern,
regulatory constraint, or stakeholder perspective that presses on the
architecture.

## Session lifecycle

- **Start a session** — opens an active session against the current
  version. All subsequent stressor commands attach to it.
- **Show** the active or a named session with its stressors and
  outcomes.
- **List** every session ever run in this scope.
- A session *closes* when the next version is saved. That save also
  triggers the automatic incidence report
  ([scoring/incidence.md](scoring/incidence.md)).

## Stressors

- **Add a stressor** with id, description, and a mandatory `--affects`
  list of **epistatic pressure surfaces**. An affect may name a **term**
  or a **type**; type entries **expand** to the set of terms it classifies
  when incidence and related analyses run (see
  [scoring/incidence.md](scoring/incidence.md)).
- **Widen or narrow affects** with `affect-add` / `affect-remove`.
  Removal refuses to empty the list — delete the stressor itself if it
  has become obsolete. The affects list also survives cascading node
  deletions: the invariant "every stressor affects ≥ 1 node" is
  preserved.
- **Attractor** — describe what state the stressor is trying to pull
  the system into (what "breaking" would look like).
- **Mark breaking** with a solution description. A breaking outcome is
  what produces new requirements that the next version must satisfy.
  Those requirements **inherit propagation** when they target types
  ([requirements.md](requirements.md)).
- **Mark surviving** when the architecture holds. Affects still stand —
  they describe where the pressure was applied, not the outcome.
- **Derive requirement** — explicitly link a requirement to a stressor,
  recording the stressor as that requirement's origin.

## Why affects is mandatory

The affects list is the join key that makes the stressor × component
incidence matrix possible. Without it, the cross-layer analyses that
surface hidden coupling, hotspots, and compound vulnerabilities would
have nothing to pivot on.
