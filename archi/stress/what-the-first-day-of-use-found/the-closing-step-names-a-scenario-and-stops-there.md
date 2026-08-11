---
affects: [Planner, Links, WorldDoc]
outcome: breaking
---

# The closing step names a scenario and stops there

Reach the closing step of a plan. `plan scenarios list` answers with six lines of
`fact-slug#scenario name`, and the anchored gate refuses, naming the ones with no link.
Neither prints the Gherkin. The operator cannot tell from the output whether a scenario is
new, already anchored and live, or anchored and drifted, and has to open three fact files
and cross-read the journal to find out. Then they compose the `link add` command by hand,
quoting a name with spaces in it.

## Attractor

The step that closes the plan is the one place the wing meets the operator at full force,
and it hands them a list of identifiers. Work that the tool already knows — which
scenarios are anchored, which drifted, what the exact ref string is — is left for a person
to reconstruct. `the-refusal-is-an-ordered-continuation` settled this shape for removal:
a refusal renders the commands that clear it. The closing step never got the same
treatment.

## Resolution

The closing step renders what it already knows: derived `the-closing-step-hands-back-the-work`.
The Gherkin whole, the link state per scenario, and a runnable `link add` with the ref
quoted. Same shape as `the-refusal-is-an-ordered-continuation`, applied to the one moment
the wing meets the operator at full force.
