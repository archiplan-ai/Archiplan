---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# The empty block asks only where a condition is owed

The close refuses an empty closing block only when a task node still owes a condition: one
that is neither classified `Data` nor named in `archi/world/.worldignore`. The refusal names
those nodes, not the plan. A plan whose every task node is carried data or declared internal
closes on an empty block and says nothing, because there was never a condition to record.

## System Context

The check read emptiness alone, so it fired on a plan that owed nothing and sent the author
to `archi world add` for a node that can never take a fact: `Data` is excluded from the
world by type, which `an-internal-element-says-so` states and `version save` already
enforces. The advice was not merely useless — it was the opposite of the design, and an
author who followed it would have written a condition about a payload.

The exclusion already exists and is already trusted. `version save` refuses on an element no
condition reaches, and it excludes `Data` by type and `.worldignore` by declaration before
it asks. Reading the same two exclusions here makes one rule serve both gates, where two
readings of "covered" had disagreed.

What this does not catch is a task cut onto the wrong node — a payload named where the
behaviour belongs to whoever reads it. The check goes quiet there instead of misdirecting,
which is the whole of the improvement. Naming the right node is the author's, and the plan
skill says how.

## Satisfy

`Planner` (computes the owing set from the task nodes, excluding `Data` and the declared
internal, and refuses the close only on a non-empty one, naming those nodes).

- test — a plan whose every task node is `Data` closes on an empty block with no refusal
- test — the same for task nodes named in `.worldignore`
- test — a plan with one behaviour node no fact covers refuses, and the refusal names that
  node
- test — the refusal names the owing nodes, not the plan
- test — a non-empty block closes as it does today
- test — a tree with no world facts at all closes as it does today
