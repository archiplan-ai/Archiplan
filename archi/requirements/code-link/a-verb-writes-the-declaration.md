---
kind: functional
origin: intent
satisfied-by: [Planner, PlanFile, Links]
deferred:
---

# A verb writes the declaration

`archi plan task <id> link add --symbol <anchor> --answers <ref> --proved-by <anchor>`
appends one entry to that task's declaration file. Every argument is required. The verb
resolves all three before it writes: the symbol and the test against the tree, the ref
against the model and the requirement set. It refuses an entry it cannot resolve, naming
which of the three failed and what it looked for, and writes nothing. A repeated identical
entry reports that it stands and appends no second copy. Several entries land in one
`archi batch -`, which fails at the first refusal and leaves the entries before it in place.

## System Context

The declaration was a file somebody typed. Typing it puts a parser between the writer and
the record, and the parser speaks after the writer has gone: a task agent that misplaces a
quote learns nothing, because the refusal reaches the orchestrator hours later. Resolving at
the moment of writing puts the refusal in front of the actor who can fix it, in the same
breath as the mistake.

It also removes the format from the surface a writer can get wrong. Nobody types TOML, so
TOML cannot be malformed, and the shape cannot drift from what the reader accepts because
one side writes what the other reads.

The three arguments stay three because they are three different claims: this code exists,
it answers that, and this test says so. Collapsing the test into a second entry was
considered — a test is a link like any other — and refused for now: the pair and its proof
are written in one act by one actor, and splitting them into two independent rows loses
which proof belongs to which pair.

## Satisfy

`Planner` (the verb, its arguments and its refusals). `PlanFile` (the entry appended to the
task's declaration file). `Links` (resolves the two anchors and the ref before the write).

- test — the verb appends one entry and the wave then closes on it
- test — a symbol that resolves to nothing refuses, naming the symbol, and writes nothing
- test — the same for a test that resolves to nothing, and for a ref that names neither an
  element nor a requirement
- test — the refusal says which of the three failed
- test — a repeated identical entry appends no second copy and says the entry stands
- test — several entries land through `archi batch -`, and a refusal mid-batch leaves the
  earlier entries written
- test — the verb refuses outside a started wave, naming the lifecycle step that opens one
