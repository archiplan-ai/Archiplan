---
covers: [Links, Links.Grader, DocsCompiler]
sources: []
uses: [why-a-design-was-chosen-lives-in-one-person-s-memory]
---

# A design written apart from the code falls behind it

The code moves faster than the document that describes it. Every change to the code is a
change somebody chose to make, and every matching change to the document is a chore
somebody has to remember; the first happens because the work demands it, the second
happens when there is time. So the gap opens from the first week and never closes. Heavy
specifications get written and then abandoned for exactly this reason — not because the
writing is hard, but because keeping two separated things in step is work nobody does
twice. The behavior follows: the two have to be joined by something a machine recomputes,
so that falling behind is a state a tool reports rather than a state a person notices too
late.

## What people do instead

The document is left to rot and the code is read in its place: whoever needs to know what
the system does opens the source and works it out again, and the writing stays for
onboarding or for show. Where the document still has to be right, somebody walks it
against the code by hand before a release — an afternoon each time, and the first thing
dropped when the release is late. Both cost the same thing twice: the hours spent
re-deriving what was already written down, and the decisions taken from a page that has
been wrong for months with nobody able to say which parts.

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

Whether reporting drift is enough in practice is not observed. A report still needs a
person to act on it, and that person is the one who did not update the document in the
first place, so the question stays open until somebody watches a real project drift and
sees whether the report gets acted on.
