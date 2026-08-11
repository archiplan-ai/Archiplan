---
kind: functional
origin: stressor(the-closing-step-names-a-scenario-and-stops-there)
satisfied-by: [Planner, Links]
deferred:
---

# The closing step hands back the work

The closing step prints each collected scenario whole — its Gherkin, not its name — with
the state of its link beside it: unanchored, anchored and clean, or anchored and drifted
with the side that moved. Each unanchored line carries the `archi link add` command with
the spec ref already quoted, so the operator supplies only the symbol. `plan verify`
answers the same way on demand.

## System Context

`the-refusal-is-an-ordered-continuation` settled this shape once: a refusal that knows
what stands in the way renders the commands that clear it. The closing step knows all of
it — which scenarios were collected, which carry links, how those links grade, and the
exact ref string with its spaces — and prints a list of identifiers instead. The operator
then opens three fact files and cross-reads the journal to rebuild what the tool already
computed, and hand-quotes a name with spaces in it.

This is the one moment where the wing meets the operator at full force, and it is the
moment the tool is least helpful.

## Satisfy

`Planner` (the closing render: the Gherkin, the per-scenario link state, and the ready
command; the same on `plan verify`). `Links` (the fold the state is read from).

- test — the closing step prints the Gherkin of every collected scenario
- test — an unanchored scenario prints a runnable `link add` with the ref quoted
- test — that printed command runs as written and anchors the scenario
- test — an anchored clean scenario prints as clean and carries no command
- test — an anchored drifted scenario names the side that moved
- test — `plan verify` prints the same three states before the last wave closes
