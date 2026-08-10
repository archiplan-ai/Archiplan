---
kind: functional
origin: stressor(the-fact-retires-under-a-running-plan)
satisfied-by: [DocMint]
deferred:
---

# Retirement refuses a plan in flight

`world rm` pre-flights a third thing beside the `uses` dependants and the link journal:
a plan with open lifecycle whose task carries this fact. It refuses, naming the plan and
the task. The operator closes the plan, repins it, or drops the fact from the task
first — and the removal then proceeds.

## System Context

`req rm` already refuses while a plan owns the slug, for the same reason: a plan in
flight is a promise that the spec under it holds still, and pulling a record out from
under an open wave turns a refusal at the right moment into rework at the wrong one. The
world verb inherits that rule rather than inventing an exception, so an operator who
knows one removal knows both. All three pre-flights report together in one message: a
person meets one refusal listing everything that stands on this fact, not three refusals
in a row.

## Satisfy

`DocMint` (reads the plan records beside the doc tree and the folded link set; the
refusal lists `uses` dependants, stranded links and blocking plans in one message).

- test — `world rm` refuses while an open plan's task carries the fact, naming plan and task
- test — the same fact retires once that plan is completed
- test — a fact carried only by a completed plan retires in one call
- test — dependants, links and plans appear together in a single refusal message
- test — the refusal exit code matches the one `req rm` uses for the same case
