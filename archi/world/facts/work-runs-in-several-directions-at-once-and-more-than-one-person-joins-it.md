---
covers: [Seats, Seats.Registry, Seats.Mint, Seats.Guard, Seats.Landing, Sessions]
sources: []
uses: []
---

# Work runs in several directions at once and more than one person joins it

A person holds three pieces of work at the same time and wants to move all three. That is
only possible when each one has a copy of the design to itself: without separate copies
the three collapse into one queue. A second person cannot join that queue at all, because
putting two sets of edits back together by hand is the worst job in the day. The behavior
follows from the queue: what a person cannot do in parallel they do in sequence, and a
design that assumes one writer makes the sequence mandatory rather than chosen.

## What people do instead

The work is done one piece at a time. A second idea waits in a note until the first piece
lands, and the note goes stale; an urgent change jumps the queue and the half-finished
piece is unpicked and set aside, then picked up days later by somebody who has to work
out where it stood. A second person either waits for the first to land or works from a
copy and merges by hand at the end — an evening spent reading two sets of edits and
deciding which one was meant, with the mistakes found later. Both are paid in
wall-clock time: the throughput of the whole is the throughput of one.

## Scenarios

### A second piece of work starts while the first is unfinished

Given a person has a piece of work in progress
When they start a second one that the first does not touch
Then each piece has its own copy of the design and its own current plan
And neither piece sees what the other has not saved

### A second person joins work already under way

Given a piece of work in progress that one person holds
When a second person starts their own piece against the same design
Then the record names who holds which copy
And putting the two back together is a ceremony rather than a hand merge

## Open questions

Whether a second person has in fact joined is not yet observed: the possibility was built
before anybody used it. The count of pieces of work run in parallel is also not measured.
