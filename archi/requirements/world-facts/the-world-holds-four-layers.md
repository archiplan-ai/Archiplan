---
kind: functional
origin: intent
satisfied-by: [WorldDoc, DocsCompiler, DocMint]
deferred:
---

# The world holds four layers

`archi/world/` holds four folders, and the folder decides what a file is. `facts/` holds
the strict record: the condition, the workaround, the scenarios, the three lists — `check`
holds every one of them. `hypotheses/` holds a claim somebody means to settle and has not.
`notes/` holds what was seen or heard and not yet shaped into either. `resources/` holds
raw material and is never parsed. A file in the three loose folders needs a name and its
prose and nothing else; `check` reads them only to resolve what a fact's `sources` names.
Four is the whole count: a file that sits in no layer is a located error, whether it lies
loose in `archi/world/` or inside a folder that is not one of the four, and the refusal
names the four by path.

## System Context

The world began as one folder of strict records, and the strictness is what makes a fact
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

Refusing the fifth folder is what makes the other three sentences true. A rule that reads
the four by name and walks past everything else does not hold four layers — it holds four
layers and an unbounded remainder that no rule describes. The loose file at the top was
already refused, so the top of the world was closed and one level down was open: a folder
somebody made in a hurry took files, kept them out of every reading, and reported nothing.
The cost is not disorder for its own sake. It is that a source resolves against the world,
so material parked outside the four is material a fact cannot name, sitting in the one
place its author believed it could.

## Satisfy

`WorldDoc` (a file's folder is what it is; the three loose folders need a name and prose).
`DocsCompiler` (the strict schema applies to `facts/` alone; the loose folders are read for
their existence and their names; a file under any other folder is refused by path).
`DocMint` (`world add` mints into `facts/`).

- test — a file under `facts/` missing its workaround or its scenarios is a located error
- test — a file under `notes/` with a name and one paragraph passes
- test — a file under `hypotheses/` with a name and one paragraph passes
- test — a file under `resources/` is never parsed and never reported
- test — `world add` mints under `facts/`
- test — a tree with none of the four folders behaves exactly as it does today
- test — a file under a folder that is not one of the four is a located error naming the
  four, at any depth below `archi/world/`
- test — the refusal is the same one the loose file at the top already raises, so the two
  never disagree about what the layers are
