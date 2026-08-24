---
kind: functional
origin: stressor(the-closing-step-names-a-scenario-and-stops-there)
satisfied-by: [Planner, Links]
deferred:
---

# The closing step hands back the work

The closing step prints each collected scenario whole — its name and its steps, not the
name alone — and what stands under it. A scenario nothing anchors is named new, and
carries the `archi link add` command with the spec ref already quoted, so the operator
supplies only the symbol. A scenario something anchors names the file and symbol it
anchors, and asks the operator to read the two against each other and repair whichever is
wrong; when the digests disagree it also names the side that moved. `plan verify` answers
the same way on demand.

## System Context

`the-refusal-is-an-ordered-continuation` settled this shape once: a refusal that knows
what stands in the way renders the commands that clear it. The closing step knows all of
it — which scenarios were collected, which carry links, how those links grade, and the
exact ref string with its spaces — and prints a list of identifiers instead. The operator
then opens three fact files and cross-reads the journal to rebuild what the tool already
computed, and hand-quotes a name with spaces in it.

This is the one moment where the world meets the operator at full force, and it is the
moment the tool is least helpful.

## Satisfy

`Planner` (the closing render: the Gherkin, the per-scenario link state, and the ready
command; the same on `plan verify`). `Links` (the fold the state is read from).

- test — the closing step prints the name and every step of each collected scenario
- test — a scenario nothing anchors is named new and prints a runnable `link add` with the ref quoted
- test — that printed command runs as written and anchors the scenario
- test — an anchored scenario names the file and symbol it anchors, and asks for the re-read
- test — an anchored scenario whose digests disagree also names the side that moved
- test — `plan verify` answers the same way before the last wave closes
