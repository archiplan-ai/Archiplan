# Fault trees — why an episode happened and what to fix

An episode (see [`episodes.md`](episodes.md)) groups the incidents of one failure
event; a **fault tree** explains it — decomposing the user-facing symptom into
the root causes that produced it, and naming the **fewest fixes** that close it.

## The tree

- **Top event** — the episode's user-facing symptom (the highest-severity /
  most-downstream incident).
- **Gates** — the decomposition logic: **AND** (all children required) / **OR**
  (any child suffices).
- **Basic events (leaves)** — the per-incident root-cause verdicts already
  produced. The tree **composes** them; it does not re-derive them.

## What the tree yields

| Output | Meaning |
| --- | --- |
| **Minimal cut sets** | the smallest sets of root causes that together produce the top event — the actionable fix targets ("fix these two and the event cannot recur") |
| **Importance ranking** | which root cause sits in the most cut sets — fix priority |
| **Cascade** | the directed path the failure propagated along (upstream cause → downstream victim) |

## Causal direction must be earned

Direction between incidents is **oriented**, never assumed:

- the **service-dependency graph** (from the Context Manager) and **trace
  evidence** orient cause vs. victim;
- a causal or grouping claim must be **grounded** — a cascade edge requires trace
  evidence, a shared-cause grouping requires code-overlap evidence; an
  unsupported link is **not asserted**.

A wrong causal edge is worse than none.

## Cost

Synthesis reasons over many incidents and is gated like any agent run: bounded by
the spend governor and triggered only when an episode is large enough to be worth
it (see [`harness.md`](harness.md)).

## Output

The episode and its cut-set fixes are the **primary Task Tracker material** — one
item for the blast radius and the fewest fixes, not one per incident — idempotent
and recurrence-aware (see [`incident-output.md`](incident-output.md)).

## Requirement

Each episode must be synthesized into a **fault tree**: the top symptom
decomposed through AND/OR gates to the per-incident root causes as leaves,
yielding **minimal cut sets** and an **importance ranking**. Causal direction
must be oriented by the dependency graph and trace evidence and grounded in
evidence; synthesis must be cost-gated.

## Questions

- Gate inference: how confidently AND vs. OR can be distinguished from evidence.
- A probabilistic layer: ranking cut sets by the leaves' confidence.
- Whether the tree updates incrementally as an episode's incidents arrive, or is
  synthesized once the episode settles.
