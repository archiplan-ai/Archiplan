---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# Scenarios close the plan

Scenarios never become tasks: they are verified end to end as the exit ceremony, not
implemented as units. After the last wave closes, `plan next` prints the scenarios
block as the final step and latches `scenarios_displayed`; one more `plan next` latches
`scenarios_closed`, prints `DONE`, and the plan is Completed. `plan reset` unlatches
both so the cycle can run again; a plan with an empty block skips the step and closes
directly. Where the block comes from is `the-plan-closes-on-the-world-s-scenarios`:
the plan collects it from the world facts covering its nodes, and authors none of its
own.

## System Context

End-to-end verification needs a home that is execution-shaped: the spec's verification
bullets prove single claims, while a scenario walks a path through many. The latch pair
is ordered — closed without displayed is a structural error a verb will refuse. The
block was free text on the plan envelope until the world landed. It was decoupled
from the spec because pinning a story to one element would have lied about its scope,
and a world fact keeps that property: `covers` is a list, so a scenario that walks four
nodes names four. This claim owns the ceremony and the latches; it no longer owns where
the stories live.

## Satisfy

`Planner` (the two latches ride the advance verb, reset clears them, the empty case
skips).

- test — plans::the_lifecycle_captures_gates_and_latches_scenarios
