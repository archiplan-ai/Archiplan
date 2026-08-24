---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# The plan names itself

The planning skill derives the plan's name from the problem statement and asks nobody. A
name the user volunteered is used as given; a name that collides with a standing plan
derives another, still without a question. The poll tool is reserved for choices that
change the work — the stack, the infrastructure, continuing a standing unit — and a name
changes nothing: it is an address, not a decision.

## System Context

Step 1 of the planning skill opened every planning session with a poll about the name —
"automation, or a free-text field of your own" — and the collision case asked a second
one. Both interrupt the operator with a choice nobody cares about, ahead of the choices
the same skill genuinely owes them. The house rule says ask by options where answers have
shapes; it never said manufacture a question where no answer matters.

## Satisfy

`AgentBrief` (step 1 derives, uses a volunteered name as given, re-derives on collision,
and puts no name question to the poll tool). `Scaffold` (installs the page byte-equal,
like the rest).

- test — the embedded plan skill derives the name itself and its step 1 names no poll
- test — a volunteered name is used as given, and a collision derives another name
  without a question
