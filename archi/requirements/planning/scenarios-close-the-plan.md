---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Scenarios close the plan

Scenarios are free-text user stories on the plan envelope, deliberately decoupled from
spec requirements: a story crosses many requirements across many nodes, and pinning it to
one element would lie about its scope. They never become tasks. After the last wave
closes, `plan next` prints the scenarios block as the final step and latches
`scenarios_displayed`; one more `plan next` latches `scenarios_closed`, prints `DONE`,
and the plan is Completed — the stories are verified end to end as the exit ceremony, not
implemented as units. `plan reset` unlatches both so the cycle can run again; a plan with
no scenarios skips the step and closes directly.

## System Context

End-to-end verification needs a home that is execution-shaped: the spec's verification
bullets prove single claims, while a scenario walks a path through many. The latch pair
is ordered — closed without displayed is a structural error a verb will refuse.

## Satisfy

`Planner` (the two latches ride the advance verb, reset clears them, the empty case
skips).

- test — plans::the_lifecycle_captures_gates_and_latches_scenarios
