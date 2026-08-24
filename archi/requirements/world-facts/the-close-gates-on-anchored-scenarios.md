---
kind: functional
origin: stressor(the-latch-says-displayed-not-passed)
satisfied-by: [Planner, Links]
deferred:
---

# The close gates on anchored scenarios

`scenarios_closed` refuses while a scenario in the collected block carries no link to
code, naming the scenarios that are unanchored. Archi never runs the scenario: the gate
proves the block is attached to something a runner executes, and the running happens where
runs belong. `plan reset` clears the latch as it always did.

## System Context

The whole case for moving the stories out of the plan was that Gherkin is executable where
free text was not, and the ceremony consuming it proved only that bytes reached a terminal
— the same receipt the free text gave. Archi has no runner and should not grow one: it
recomputes hashes, it does not execute code. What it can prove is attachment, and the plan
already owns that machinery — the asserted-coverage gate reads the folded link set for
exactly this kind of question. A latch on an anchored block is a real claim: somebody
authored the edge, and the edge is verifiable by a run in CI.

## Satisfy

`Planner` (the latch consults the folded links for every collected scenario and refuses
with the unanchored list). `Links` (serves the fold, as it does for the coverage gate).

- test — a block whose scenarios all carry links latches closed
- test — one unanchored scenario refuses the latch and is named
- test — an empty block closes directly, as it does today
- test — `plan reset` clears the latch after a refusal
- test — the refusal exit code matches the plan's other gate refusals
