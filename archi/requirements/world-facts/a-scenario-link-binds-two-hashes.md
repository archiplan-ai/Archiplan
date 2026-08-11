---
kind: functional
origin: intent
satisfied-by: [Links, Links.Canonizer, Links.Grader]
deferred:
---

# A scenario link binds two hashes

A link from a scenario to code records two digests: one over the scenario as the grammar
parsed it, one over the code item, as every link already does. `link verify` fails the link
when either digest moves, and says which side moved. `link repin` binds the pair again. The
address stays the scenario name; the digests are what decides whether the pair still holds.

## System Context

`the-scenario-is-the-address-not-the-step` settled that a step is not addressable, and that
stands: the address is still the scenario, and renaming it is still what breaks resolution.
What that claim also said — that rewording a step leaves the link untouched — is now wrong,
and the operator overruled it for a concrete reason. The pair is written together: the
scenario says what must happen, the code is generated to answer it. Afterwards either side
can move alone. Code that changed under an unchanged scenario is the common case and the
dangerous one, because everything reads green while the promise and the answer have parted.

So the link is not a pointer, it is a witnessed pair, and a witness that no longer matches
what it witnessed is a failure rather than a note. The cost is real and was weighed: prose
edits to a step will fail links over untouched code, and the operator will repin. That is the
price of the pair being meaningful at all.

## Satisfy

`Links.Canonizer` (a digest over the parsed scenario — feature, scenario name, step keywords
and step text — beside the code digest it already computes). `Links.Grader` (fails a scenario
link when either digest moves and names the side). `Links` (`repin` binds the pair again).

- test — a link over an unchanged pair verifies clean
- test — a reworded step fails the link and the message names the scenario side
- test — a changed code item fails the link and the message names the code side
- test — both sides moved: the failure names both
- test — `link repin` binds the new pair and the next verify is clean
- test — a link over an element path, not a scenario, grades exactly as it does today
