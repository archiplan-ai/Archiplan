---
covers: [Links, Links.Grader, DocsCompiler]
sources: [archi/requirements/modeling-language/modeling-language.md]
uses: [why-a-design-was-chosen-lives-in-one-person-s-memory]
---

# A design written apart from the code falls behind it

The code moves faster than the document that describes it. Every change to the code is a
change somebody chose to make, and every matching change to the document is a chore
somebody has to remember; the first happens because the work demands it, the second
happens when there is time. So the gap opens from the first week and never closes. The
operator has written a heavy specification twice and abandoned it both times for exactly
this reason — not because the writing was hard, but because keeping two separated things
in step is work nobody does twice. The behavior follows: the two have to be joined by
something a machine recomputes, so that falling behind is a state a tool reports rather
than a state a person notices too late.

## What kills this

The document and the code stop being able to diverge — a change to one is impossible
without a change to the other, not merely expected to come with it. A signal after the
fact is not enough: a report that says "these have drifted" still needs a person to act,
and that person is the one who did not update the document in the first place.

## Scenarios

### The code moves and nobody updates the writing

Given a written claim with files recorded against it
When those files change and the claim is left alone
Then the drift is reported against that claim by name
And the report names the files that moved

### A claim is written that no code answers

Given a written claim with no files recorded against it
When somebody asks what is unaccounted for
Then the claim is named as promised and unbuilt

## Open questions

Whether reporting drift is enough in practice is not observed. The operator says a signal
alone will not do, and the tool ships a signal, so the two disagree until somebody watches
a real project drift and sees whether the report gets acted on.
