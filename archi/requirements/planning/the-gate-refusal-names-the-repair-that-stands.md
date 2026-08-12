---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# The gate refusal names the repair that stands

`archi plan next` blocked on coverage names the repair that exists: hand-author the missing
link with `archi link add`. It names no list of candidates to review and no `link confirm`,
because capture proposes nothing to review. One test reads that refusal, so the text cannot
drift from the mechanism again without a red suite.

## System Context

The refusal is read at the one moment a person is stuck, so a continuation that does not
exist costs more than silence would. This one survived three units past the mechanism it
describes: capture stopped proposing candidates, and the wording that sent the reader to
`archi link ls --evidence` and `archi link confirm` stayed.

The briefing carries a guard against this phrasing now, and the guard reads the skills
only. The tool's own text is where the phrasing started, and nothing read it.

## Satisfy

`Planner` (the coverage refusal of `plan next` and the prose that documents it).

- test — the coverage refusal of `plan next` names `archi link add` as the repair
- test — the coverage refusal names neither `link ls --evidence` nor `link confirm`
