---
kind: functional
origin: intent
satisfied-by: [WorldDoc, DocsCompiler, DocMint]
deferred:
---

# The world holds four layers

`archi/world/` holds four folders, and the folder decides what a file is. `facts/` holds
the strict record: the condition, what kills it, the scenarios, the three lists — `check`
holds every one of them. `hypotheses/` holds a claim somebody means to settle and has not.
`notes/` holds what was seen or heard and not yet shaped into either. `resources/` holds
raw material and is never parsed. A file in the three loose folders needs a name and its
prose and nothing else; `check` reads them only to resolve what a fact's `sources` names.

## System Context

The wing began as one folder of strict records, and the strictness is what makes a fact
worth its place: a condition, an opposing observation that would end it, and behaviour that
follows. That same strictness is why nobody writes one on the way past. Material arrives
loose — a thing somebody noticed, a claim they mean to test, a transcript — and with one
folder it either got forced into the strict shape or stayed out of the tree.

The folder decides the kind, not a field. That was settled once already when `kind` left the
frontmatter: a field would have to be read by every check, and a check would then apply four
sets of rules by value. Four folders apply them by place, and the loose three carry no
schema to keep true.

The layers are not a ladder anybody has to climb. A note may sit forever. A hypothesis may
die unsettled. What the arrangement buys is that a fact can name where it came from without
reaching outside the world.

## Satisfy

`WorldDoc` (a file's folder is what it is; the three loose folders need a name and prose).
`DocsCompiler` (the strict schema applies to `facts/` alone; the loose folders are read for
their existence and their names). `DocMint` (`world add` mints into `facts/`).

- test — a file under `facts/` missing its killer or its scenarios is a located error
- test — a file under `notes/` with a name and one paragraph passes
- test — a file under `hypotheses/` with a name and one paragraph passes
- test — a file under `resources/` is never parsed and never reported
- test — `world add` mints under `facts/`
- test — a tree with none of the four folders behaves exactly as it does today
