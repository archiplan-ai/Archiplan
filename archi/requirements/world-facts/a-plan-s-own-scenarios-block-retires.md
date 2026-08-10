---
kind: functional
origin: stressor(the-standing-plan-loses-its-stories)
satisfied-by: [Planner, PlanFile]
deferred:
---

# A plan's own scenarios block retires

No verb reads a plan's `scenarios.md`, no verb deletes it, and no finding fires for one.
A plan written before the wing simply has no scenarios, and that is the end of it. For a
plan created after, an empty collected block is not a valid close: `plan next` refuses the
final latch and says that no world fact covers any node this plan holds a task for.

## System Context

Twenty-six plan folders hold stories written under the old rule and converting them is
judgement, not mechanics — naming the condition under a story is the whole thing the wing
asks for, and a converter would have produced facts with empty sources and a sentence
lifted from a story. So history is left exactly as it is and stays silent, with no finding
to mute and no work to schedule. What the rule buys instead is applied forward: a new plan
that can close with nothing is a plan whose nodes nobody has said anything true about, and
that is the state the refusal names.

## Satisfy

`Planner` (ignores `scenarios.md` entirely; refuses the closing latch on an empty collected
block for a plan minted after the wing, and closes a pre-wing plan as it always did).
`PlanFile` (the old block stays on disk, read by nobody).

- test — a pre-wing plan closes with no block and raises no finding
- test — a post-wing plan with an empty collected block refuses the final latch
- test — the refusal says that no fact covers any of the plan's task nodes
- test — the same plan closes once one covering fact exists
- test — no verb writes or deletes `scenarios.md`
