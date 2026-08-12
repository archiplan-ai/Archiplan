---
kind: functional
origin: stressor(the-source-lives-outside-the-tree)
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# A source is reachable and lives in the world

A `sources` entry names a file under `archi/world/` — a note, a hypothesis or a resource —
and `check` resolves it. An entry that resolves to nothing is a located error. An entry
pointing outside the world is a located error, and the message says why: the spec is what
the world conditions, so a fact grounded in a requirement grounds itself in what it
explains. `sources` may be empty, and an empty one is the ungrounded state as before.

## System Context

The field once accepted any path in the tree and any external locator, and both were
mistakes made for the same reason: to let a fact look grounded when nothing had been
observed. The migration used the first — every fact it wrote named the intent it was lifted
from — and that made the tree quiet about facts nobody had checked. The external form made
the field unfalsifiable: a ticket id or a recording nobody here can open is a claim about
evidence, not evidence.

A source that cannot be reached is not a source. So the material comes into the world or the
field stays empty, and an empty field says the true thing: nobody has grounded this yet. That
is not a defect to hide; `world_ungrounded` reports it, and it is the world's own measure of
how much of it rests on somebody having looked.

The cost is real and is taken deliberately: an interview that lives in a drive is transcribed
into `resources/` or it does not count. Transcribing is work, and work is the price of the
field meaning anything.

## Satisfy

`WorldDoc` (`sources` entries are paths under `archi/world/`). `DocsCompiler` (resolves each
entry; a miss and a path outside the world are located errors, the second naming why).

- test — an entry naming a file under `notes/`, `hypotheses/` or `resources/` resolves
- test — an entry naming a file that does not exist raises a located error
- test — an entry naming a path outside `archi/world/` raises a located error that says why
- test — an entry carrying a URI scheme raises the same error
- test — an empty `sources` passes and reports `world_ungrounded`
- test — the four standing facts carry no source and report ungrounded
