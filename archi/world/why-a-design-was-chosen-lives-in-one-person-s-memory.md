---
covers: [DocsCompiler, DocMint]
sources: [archi/requirements/modeling-language/modeling-language.md]
uses: []
---

# Why a design was chosen lives in one person's memory

The reasons behind a shape — what was traded, what was refused, what the pressure was —
stay with whoever made it. They are not in the files, because the files say what the
system does and not why it does that instead of the other thing. Memory decays, so the
person who made the shape reconstructs it later from what they can still recall, and
everybody else asks them. The operator has kept a whole design that way and watched it
fade. The behavior follows: the reasons have to be written down as records that a tool
reads and holds against the shape, or the only copy leaves when the person does.

## What kills this

Code that carries the semantics of the decisions behind it — a reader looks at the source,
across every repository the system spans, and sees why the shape is this one and not the
other. The operator does not expect that to arrive, and notes that people invented notations
and decision records precisely because it has not. But it is an observable state: the day a
reader can answer "why" from the code alone, the written reasons are duplication.

## Scenarios

Feature: The reasons behind a shape outlive the person who chose it

  Scenario: Somebody asks why months after the choice
    Given a shape whose reasons were recorded when it was chosen
    When a reader who was not there asks why it is this way
    Then the record answers with the trade and what it cost
    And nobody has to ask the person who made the choice

  Scenario: The person who made the choice has forgotten
    Given a shape chosen a year ago by a person still here
    When that person cannot recall what they traded
    Then the record answers in their place

## Open questions

Whether a reader who was not there actually finds the record is not observed: so far the
only readers have been the person who wrote it and the assistants they drive.
