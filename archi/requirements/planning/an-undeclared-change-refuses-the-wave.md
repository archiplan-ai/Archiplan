---
kind: functional
origin: intent
satisfied-by: [Planner, Links.Capture]
deferred:
---

# An undeclared change refuses the wave

`archi plan next` refuses while a task in flight has written no declaration file, or has
written one that declares nothing. The refusal names the task and the path it owes. This is
the presence check, and it is the only one that names a task. What the file holds is
answered by a second gate, on the delta rather than on the task
(`the-file-in-the-delta-is-the-unit-the-gate-demands`).

## System Context

The gate is the file, and the file alone. It was drawn wider once — every symbol in the
wave's delta had to be named by some declaration — and the reach is what killed it. Applied
to the wave that built it, the wider rule demanded thirty entries for two changed files: three
for the code, and twenty-seven for tests, test helpers and the string constants inside them.
A test answers no part of the spec, and a fixture constant answers nothing at all, so the
rule was asking for a field that does not exist for the thing it was asking about.

What this check gives up is caught by the other one: the delta's files are demanded of the
wave as a whole, so a task that declares one pair of the ten it owes closes nothing unless
every file the wave moved is named by somebody. What stays given up is the symbol: inside a
named file, which of its symbols an entry speaks for is the writer's call. Nothing can prove
a declaration true, so the accounting was never going to be enforced past its presence.

## Satisfy

`Links.Capture` (reports which in-flight tasks wrote no file and which wrote an empty one).
`Planner` (refuses the wave on either, naming the task and the path it owes).

- test — a wave with a task whose declaration file is absent refuses, naming the task and
  the path
- test — a file that parses and declares nothing refuses the same way
- test — a file with one entry passes this check, whatever else the delta moved
- test — the refusal names the next command
- test — a wave with no task in flight owes no file
