---
kind: functional
origin: stressor(the-file-the-model-must-write-is-refused-for-its-shape-and-the-wave-stops-on-form)
satisfied-by: [Planner, Links.Capture]
deferred:
---

# The declaration refusal repairs without guessing

A malformed declaration file is refused with the line, what was expected there and what was
found. The refusal never restates the grammar alone. The file's shape is the smallest one
that carries the claim — a symbol, what it answers, and the test — and every field is
required, because an optional field is the beginning of a file that always parses.

## System Context

The file is written by a language model and read by a strict parser, and the writer has
already returned by the time the parser speaks. So the refusal lands on the orchestrator,
who did not choose the format and holds only the sub-agent's report of what the file was
meant to say. This tree has already paid for the pattern: a plan bullet that wrapped onto a
second line was refused with a message that restated the grammar and never said the bullet
wrapped, and two rounds of guessing went into it — with a person reading.

The pressure such a refusal creates is toward loosening, and loosening is what kills the
gate. Optional fields, ignored unknown keys, a missing section downgraded to a warning: each
is locally reasonable and the sum is a file that always parses and asserts whatever it
happens to hold. The answer is not a laxer parser but a refusal that costs one read to fix,
so nobody reaches for the parser.

`refusals-name-the-continuation` already claims this for every refusal in the tool. It is
restated here because the reader of this one is not the writer of the file, which is a case
that requirement never had to cover.

## Satisfy

`Links.Capture` (parses the file and reports the line, the expectation and the finding).
`Planner` (surfaces it as a refusal that names the file, the task and the repair).

- test — a missing required field is refused with its line, what was expected and what stood
  there
- test — an unknown key is refused rather than ignored
- test — the refusal names the task and the path of the file
- test — no refusal consists only of a restatement of the grammar
- test — a file that parses with every field present is accepted with no warning
