---
covers: [Seats, Seats.Registry, Seats.Mint, Seats.Guard, Seats.Landing]
sources: [archi/requirements/worktree-parallelism/worktree-parallelism.md]
uses: []
---

# Work runs in several directions at once and more than one person joins it

A person holds three pieces of work at the same time and wants to move all three. That is
only possible when each one has a copy of the design to itself: without separate copies
the three collapse into one queue. Before the copies existed the operator lived that way —
one piece of work at a time, from the first step of the ritual to the last — and a second
person could not join at all, because putting two sets of edits back together by hand was
the worst job in the day. So the tool was for one player, and its throughput was one. The
behavior follows from the queue: what a person cannot do in parallel they do in sequence,
and a design that assumes one writer makes the sequence mandatory rather than chosen.

## What kills this

The operator ends up alone and sequential — one piece of work at a time, by preference,
with nobody else touching the same design for a year. Then a copy per piece of work buys
nothing that a single tree does not already give, and every ceremony around the copies is
overhead.

## Scenarios

Feature: Several pieces of work share one design without meeting

  Scenario: A second piece of work starts while the first is unfinished
    Given a person has a piece of work in progress
    When they start a second one that the first does not touch
    Then each piece has its own copy of the design and its own current plan
    And neither piece sees what the other has not saved

  Scenario: A second person joins work already under way
    Given a piece of work in progress that one person holds
    When a second person starts their own piece against the same design
    Then the record names who holds which copy
    And putting the two back together is a ceremony rather than a hand merge

## Open questions

Whether a second person has in fact joined is not yet observed: the possibility was built
before anybody used it. The count of pieces of work run in parallel is also not measured.
