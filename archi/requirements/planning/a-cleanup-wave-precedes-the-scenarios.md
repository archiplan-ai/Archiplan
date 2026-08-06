---
kind: functional
origin: stressor(wave-born-twins-reach-the-landing)
satisfied-by: [Planner]
deferred:
---

# a cleanup wave precedes the scenarios

When the last wave closes, `plan next` announces the cleanup stage
once — the same latch pattern the scenarios use — and only the next
`plan next` moves on to the scenarios block. The stage is recorded in
the plan's lifecycle state and moves only through the verb. A legacy
plan mid-flight keeps its meaning: the stage appears for plans that
close their last wave under the new binary.

## System Context

Parallel sub-agents with disjoint write surfaces can give birth to one
mechanism twice, and nothing between the waves and the scenarios looks
for the twins. The scenarios are the closing blessing — they must run
on the folded code, so the sweep sits strictly before them. The
briefing carries the sweep's contract; the lifecycle carries the stage.

## Satisfy

`Planner` gains the stage between the waves and the scenarios: a
`cleanup_closed` latch in the lifecycle state, announced and advanced
by `plan next`.

- test — plan_e2e: after the last wave, `plan next` prints the cleanup
  block once; the next call prints the scenarios; the one after
  completes; a plan closed before the stage existed still reads
