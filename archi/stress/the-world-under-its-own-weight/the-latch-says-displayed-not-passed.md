---
affects: [Planner, WorldDoc, PlanState]
outcome: breaking
---

# The latch says displayed not passed

Close a plan. `plan next` prints the collected block, latches `scenarios_displayed`, then
`scenarios_closed`, and the plan is Completed. Nothing ran. The whole case for moving the
stories into the world was that a Gherkin block is executable where free text was not, and
the ceremony that consumes it still only proves that bytes were printed at a terminal. A
latch records that somebody saw the scenario, which is exactly what the free text
offered.

## Attractor

The world pays the full cost of executable scenarios — a real parser, member tags, scenario
addressing, links to step definitions — and collects the same receipt the free text gave.
An operator latches past a red scenario as easily as a green one, and the property that
justified the whole move is never once exercised by the tool that demanded it.

## Resolution

The latch gates on attachment, not on a run: derived `the-close-gates-on-anchored-scenarios`.
`scenarios_closed` refuses while a collected scenario carries no link to code. Archi grows
no runner — it recomputes hashes and does not execute — so what it can prove is that
somebody authored the edge to something a runner executes. That is a real claim where a
display latch was none, and it reuses the asserted-coverage machinery the plan already has.
