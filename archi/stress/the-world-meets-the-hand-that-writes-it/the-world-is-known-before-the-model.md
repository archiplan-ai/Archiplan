---
affects: [WorldDoc, Scaffold, DocsCompiler]
outcome: breaking
---

# The world is known before the model

Start a project. The conditions are what you know first — who the person is, what they do
now, what it costs them — and there is no model yet, so `covers` has nothing to name. The
chain this world was designed around runs from the world through scenarios into nodes, and
the tool requires the nodes to exist before the first fact can be written. `check`
resolves `covers` against the live model and an unresolved entry is an error.

## Attractor

The world is unusable exactly when it is most valuable. A greenfield author writes the
model first from intuition, then back-fills facts to cover what they already built —
which inverts the whole point and turns the world into a justification layer applied after
the fact. Or they leave `covers` empty on every early fact, and every early fact reports
as an orphan on a tree where nothing is wrong.

## Resolution

An empty `covers` becomes legal: derived `a-fact-may-stand-before-the-model-does`.
`world_uncovered` keeps the gap visible and `world_orphan` narrows to the fact nothing
names and that carries no scenarios. The world joins the pattern requirements already
follow — `satisfied-by` stays open and its emptiness is a finding, not an error — instead
of demanding a model before the first observation can be written down.
