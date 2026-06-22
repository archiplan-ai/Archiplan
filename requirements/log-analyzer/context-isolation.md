# Context isolation per incident (one run per incident)

**Policy: one analysis run per incident.** Each distinct incident is grounded in
its **own** agent run, with a context assembled for that one failure — not in a
shared run that reasons over many incidents at once. This is a deliberate
granularity choice, separate from the *dedup* that produces the incidents in the
first place (see [`incident-lifecycle.md`](incident-lifecycle.md)).

## Why isolate

Analyzing many failures in a single run is cheaper but wrong on several axes; one
run per incident buys:

| Property | What isolation guarantees |
| --- | --- |
| **No cross-contamination** | reasoning about one failure never sees another's evidence, so a verdict can't anchor to the wrong incident or blur two failures into one explanation |
| **Bounded context** | each run's context is sized to a single failure — its messages, its history, the sources it needs — so it never overruns the model's context window the way an all-incidents run does as the batch grows |
| **Clean attribution** | cost, tokens, the verdict, and the cited sources attribute to exactly one incident, which is what the audit ledger and the analytics surfaces depend on (see [`harness.md`](harness.md)) |
| **Focused fetching** | the agent reads only the sources that one failure implicates, rather than everything every incident in a batch might touch |

## What the per-incident context holds

The run is given what it needs to ground **one** failure, and a controlled,
read-only slice of cross-incident signal — no more:

- the incident's representative message and its variant messages;
- its occurrence and trace context, and the build revision it occurred on;
- its own prior analysis history (so re-analysis builds on earlier verdicts);
- **just enough awareness of sibling incidents** to recognize a duplicate and
  merge rather than re-create — bounded to identities, not a dump of every other
  incident's full evidence.

Isolation is therefore not total blindness: the run sees the *existence* of its
siblings for the deduplication decision, but **reasons** over only its own failure.

## Relationship to the rest of the pipeline

1. Deduplication decides **what** the distinct incidents are.
2. **This policy** says each then gets its **own** run with an isolated context.
3. Those runs are ordered so deduplication stays correct — a later run must be
   able to see an earlier sibling's incident, so duplicates merge rather than
   re-create.

## The synthesis exception

Per-incident isolation governs **leaf derivation** — producing each incident's
verdict. It does **not** forbid all cross-incident reasoning: **episode
synthesis** ([`episodes.md`](episodes.md), [`fault-trees.md`](fault-trees.md)) is
the one deliberate place the system reads across incidents. It composes the
finished **verdicts** (the leaves) into a fault tree — it never reads another
incident's raw evidence and never re-derives or alters a leaf. Isolation holds
for analysis; synthesis is an explicit, scoped layer on top.

## Requirement

Every incident must be analyzed in its own run with a context scoped to that
single failure. A run must never depend on, or be polluted by, the evidence of
another incident — beyond the minimal sibling-identity signal required to
recognize duplicates. The sole exception is the episode-synthesis layer, which
reads finished verdicts (not raw evidence) by design.

## Open questions

- **How much sibling context is right.** Enough to dedup, not enough to bloat or
  bias — the exact slice (which siblings, how summarized) is open.
- **Shared root causes.** When two incidents share one underlying cause, isolated
  runs each investigate it independently; **relating them is the job of the
  episode layer** (see [`episodes.md`](episodes.md)), not of the per-incident
  runs. Whether leaf-level reuse/memoization is also worth it is open.
- **Cost of isolation.** N runs cost more overhead than one batched run; the
  policy trades spend for verdict quality and attribution — where that trade
  stops being worth it (very large batches) is open.
