---
kind: functional
origin: stressor(the-block-moves-under-the-plan)
satisfied-by: [Planner, WorldDoc]
deferred:
---

# The close re-reads the wing and says what moved

The block is collected at the close, from the tree as it stands, and the close names
its drift against what the tasks carried: a fact retired since the plan was authored,
a fact whose scenarios changed, and a fact that now covers a plan node and did not
before. `plan verify` reports the same drift at any point, not only at the end. The
plan stores no copy of the block.

## System Context

A plan pins a version so that code is written against a spec that holds still. The wing
is not in that pin — no doc is — so the choice was a copy or a report. A copy would put
the same scenario in two files and make the plan the second place a person edits when a
fact moves, which is the drift the wing exists to remove. A report costs nothing to keep
true and matches the verb that already exists: `plan verify` flags every task whose
obligations no longer hold, and a covering fact that moved is exactly such an obligation.
The operator then chooses — `plan repin` to adopt the new picture deliberately, or fix
the fact.

## Satisfy

`Planner` (collects at close, diffs against the fact slugs the tasks carry, prints the
drift above the block; `plan verify` runs the same diff on demand). `WorldDoc` (the one
copy of the scenario, in the file that carries its reason).

- test — a fact retired since authoring is named as drift at close
- test — a fact whose scenarios changed since authoring is named as drift
- test — a fact that began covering a plan node after authoring is named as drift
- test — no drift prints when nothing moved, and the block prints alone
- test — `plan verify` reports the same three cases before the last wave closes
- test — the plan record holds fact slugs and no scenario text
