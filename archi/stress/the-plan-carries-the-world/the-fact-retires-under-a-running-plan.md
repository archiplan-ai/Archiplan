---
affects: [DocMint, Planner, WorldDoc]
outcome: breaking
---

# The fact retires under a running plan

Run `world rm` while a plan is in flight over a node that fact covers. The removal
pre-flights two things — the `uses` dependants and the link journal — and neither is a
plan. `req rm` refuses while a plan owns the slug; the world verb has no such refusal,
because when it was written the plan did not know the world existed. The task keeps a
name that resolves to nothing, and the wave it sits in is open.

## Attractor

An implementer opens a task mid-wave and reads a covering fact that is no longer there,
or reads nothing where the condition used to be and writes the code against the
happy path. `plan verify` flags it afterwards, which is the right verb at the wrong
moment: the work is done by then, and the flag arrives as rework rather than as a
refusal at the point of the mistake.

## Resolution

The removal inherits the rule `req rm` already carries: derived
`retirement-refuses-a-plan-in-flight`. A plan in flight is a promise that the spec under
it holds still, so pulling a fact out from under an open wave refuses, naming the plan and
the task. All three pre-flights — dependants, links, plans — report in one message, so a
person meets one refusal listing everything that stands on this fact.
