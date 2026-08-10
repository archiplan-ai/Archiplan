---
affects: [Links, Gherkin, WorldDoc]
outcome: breaking
---

# The step is reworded and the link holds

Anchor a link from a scenario to its step definition, then reword the step: `Given an
operator has a unit of work in progress` becomes `Given the operator has work in
progress`. The Gherkin still parses and the runner still binds by its own pattern, but
the spec ref the journal recorded named the old text. Link kinds grade by projection —
`literal` where the exact body is the contract, `indirect` otherwise — and neither kind
was defined against a sentence of English that a person may rewrite for taste.

## Attractor

Either every rewording decays a link, and the operator learns that link drift on this
wing means nothing, or nothing decays and a step can be rewritten into a different
assertion while its link stays green. The one machine-decided edge from spec to code
grades on prose, and prose is the thing in this repository people edit most freely.

## Resolution

The addressable unit is the scenario, not the step: derived
`the-scenario-is-the-address-not-the-step`, and `scenarios-parse-or-the-check-fails` is
corrected — it promised addressable steps, which was the mistake this pressure found. A
scenario name is a name, edited deliberately and rarely, so it grades like every other
reference here. Step text stays free for anyone to reword, which is what step text is
for.
