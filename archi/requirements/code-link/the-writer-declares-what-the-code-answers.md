---
kind: functional
origin: intent
satisfied-by: [Planner, Links.Capture, Links.Journal]
deferred:
---

# The writer declares what the code answers

A task's sub-agent writes one declaration file beside the wave index, naming for each
symbol it changed the port or requirement that symbol answers. `archi plan next` refuses
while a task in flight has no such file, and while a name in one resolves against neither
the model nor the requirement set. What the file names becomes an asserted link; what it
does not name becomes nothing. Ports are named, never edges: an edge is a caller, and the
code behind a port does not know its callers.

## System Context

The writer holds the answer. `archi plan task show` hands the sub-agent its `spec_refs`
before a line is written, the sub-agent implements against them, and the current contract
then forbids it from recording which one it answered — every `link` command belongs to the
orchestrator. The knowledge is issued, used and discarded, and capture spends the rest of
the wave reconstructing it from shared words.

The reconstruction does not work. On the unit that preceded this round the two functions
carrying the whole change drew no candidate, while the helper that formats one diagnostic
string drew three, because the message holds the word `world` and the refs name `WorldDoc`.
Across the tree the count is 2036 captured candidates unread against 103 confirmed.

A file rather than a comment, for one reason that decides it: canonical tokens strip
comments in Rust and keep them everywhere else, so a comment is free in one language and
breaks every link on the file in the others. The file is also input rather than a home —
consumed by `plan next`, never read again, with the journal as the durable record.

## Satisfy

`Planner` (requires the file per in-flight task and refuses the wave without it).
`Links.Capture` (reads the declarations instead of inferring from shared terms).
`Links.Journal` (mints the declared pairs asserted, recording that a writer declared them).

- test — a wave with a task whose declaration file is absent refuses, naming the task
- test — a declared name that resolves against neither the model nor a requirement refuses,
  naming the name and the file line
- test — a declared port becomes an asserted link on the symbol that named it
- test — an edge in a declaration is refused, and the refusal names the port behind it
- test — nothing is minted for a symbol no declaration names
