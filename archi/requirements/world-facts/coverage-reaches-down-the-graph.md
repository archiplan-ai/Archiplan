---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Coverage reaches down the graph

A node is covered when a world fact names it in `covers`, when it is reachable from such a
node along the declared connection edges, or when it is a child of a covered node. On a
tree holding at least one fact, `check` reports `world_unreached` for every element outside
that set. On a tree with no facts it reports nothing at all, because a project that has not
opted into the wing is not behind on it.

## System Context

Scenarios run from the surface inward, so covering the few nodes a person touches carries
coverage through the edges already declared: two facts on the outermost services reach most
of a model. That is why a rule demanding a fact per node was refused — it would name every
lexer and canonizer in the tree and be muted on the first day, which
`the-wing-arrives-without-noise` settled. Reachability names something different: an element
that no recorded behavior can arrive at.

The finding belongs to `check` and not to the plan. Incompleteness is the normal state of
the spec phase, where findings are the worklist and shrink as the work is done —
`unsatisfied_requirement` has that exact shape. A gate at plan time would invert the order
this tool is built on: the plan projects a hardened version, and a plan that stops to demand
new spec is a plan writing spec. It would also force a repin to pick up a fact authored
after a task, which spends the pin on a document that never enters it.

## Satisfy

`DocsCompiler` (computes the covered set from the facts' `covers` by forward reachability
over connection edges and by containment, then emits `world_unreached` per element outside
it; silent when the wing is empty).

- test — an element a fact names directly is not reported
- test — an element reachable from a covered element is not reported
- test — a child of a covered element is not reported
- test — an element nothing reaches is reported by path
- test — a tree with no world facts reports nothing about coverage
- test — the finding never changes the exit code
