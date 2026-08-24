---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# Each node names the condition that needs it

The briefing carries one question wherever an agent walks the nodes — capturing the world,
drafting the model, migrating a standing project: **which outside condition stops holding
if this node is gone?** The agent drafts its own answer first and puts it to the operator
through the poll tool, the drafted condition as an option and "nothing outside reaches this
node" as the last, which routes to `.worldignore`. An answer that stands is the fact the
layer is for. A symptom cannot answer, because the symptom survives the node's removal.
The question is one sentence, word-identical in every page that carries it, and a guard
holds the copies together.

## System Context

The skill sources facts from what the operator already said, and the save gate goes quiet
at the first covering fact — so the condition a whole layer exists for, which the operator
never says aloud because it is obvious to them, is never minted. Measured live twice: a
unit-arithmetic compiler whose reason — mistakes in units are invisible to the eye and
hand review is expensive — was recorded only after the operator prodded; and a fact set
where every fact grew from the symptom that had just been found. The prod is real work the
operator does by hand in every project; this question is that prod, written down.

The gate itself cannot be sharpened: condition and symptom are indistinguishable to a
compiler, and presence-not-sufficiency is the recorded trade. The question lives with the
writer — a new candidate source that does not depend on what was spoken — and the operator
judges the answer through options, never an open ask.

## Satisfy

`AgentBrief` (the question at the three walks: the world capture, the model draft, the
migration's world pass — each drafting the answer and polling it). `Scaffold` (installs the
pages byte-equal, like the rest).

- test — the question sentence stands word-identical in the workflow skill's capture and
  model steps and in the migrate page's world pass
- test — each placement drafts the answer and puts it through the poll tool, with the
  no-outside-condition option routing to `.worldignore`
- test — the capture placement says a symptom cannot answer because it survives the node's
  removal
