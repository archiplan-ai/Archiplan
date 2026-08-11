---
kind: functional
origin: intent
satisfied-by: [Planner, Links.Capture]
deferred:
---

# An undeclared change refuses the wave

`archi plan next` refuses while the wave's delta holds a changed symbol that no declaration
names. The refusal lists each such symbol with the task whose outputs claim its file. A
changed file no task claims stays what it is today, a leftover note, because no declaration
was owed for it.

## System Context

Requiring the file to exist is a formality: an empty file satisfies it. The gate that
matters is the other half, and it is the half that closes the failure this whole round
exists for. Under inference a ref that shared no word with any changed symbol was neither
proposed nor pressed — it left the checklist silently and the wave closed reporting complete
coverage having never asked. Reading the requirement from the delta instead of from a word
match makes silence impossible: the delta already knows every symbol that moved.

The two halves are stated apart because they fail apart. A missing file is one refusal
naming one task. An undeclared symbol is a list, and the list is the work.

## Satisfy

`Links.Capture` (computes the undeclared set from the delta it already computes).
`Planner` (refuses the wave on a non-empty set and names each symbol with its task).

- test — a wave where one changed symbol is declared and another is not refuses, naming only
  the second
- test — the refusal names the task whose outputs claim the file
- test — a changed file no in-flight task claims stays a leftover and does not refuse
- test — declaring the missing symbol clears the refusal and the wave closes
- test — a wave whose delta is empty closes with no declaration owed
