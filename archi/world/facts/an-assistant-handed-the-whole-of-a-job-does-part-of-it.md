---
covers: [Planner]
sources: []
uses: []
---

# An assistant handed the whole of a job does part of it

Give an assistant everything a large piece of work contains and it will not do all of it.
Some of the list is skipped outright. Some is read as licence to wander past the edge of
what was asked, into work nobody ordered. Neither failure announces itself: what comes
back looks finished, and the gap is found later by whoever knows what was supposed to
happen. The behavior follows from the size of what arrives: work has to be handed over in
portions small enough that nothing in the portion can be missed, each portion named before
it starts and closed before the next one opens, so what was not done is visible while it
still costs little to do.

## What people do instead

The list is kept in a file and handed over entire, because handing over a file is one
action and cutting the work into portions is many. Then somebody reads what came back
against what was asked, notices the gaps, and asks again. That reading is the only thing
standing between a skipped item and a shipped one, and it is done by a person who has to
remember what was on the list — so what suffers is the quality of what gets built, and it
suffers silently.

## Scenarios

### Work arrives in portions and the next does not open early

Given a piece of work cut into portions
When one portion is handed over
Then only that portion is described
And the next portion does not open until this one is closed

### What a portion was for is readable while the portion runs

Given a portion in flight
When somebody asks what it must satisfy
Then the obligations it carries are named with it

## Open questions

How much is skipped when the whole arrives at once is not measured, and neither is how
much of the wandering is caught by the reading that follows.
