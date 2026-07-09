---
affects: [Scaffold, SourceTree]
outcome: breaking
---

# A re-run clobbers the tree

A month into a project, an agent's session notes still open with "run `archi init`
first" — it obeys. An init that writes its artifacts unconditionally replaces
`model.arch` with the starter comment and resets `CLAUDE.md` to the stock block;
the morning goes to `git checkout` archaeology — or, on a tree not yet committed,
to reconstruction from memory.

## Attractor

Init is the one verb whose habitat is "probably not a project yet" — it cannot ask
the tree for permission the way every other verb does, and humans and agents alike
re-run setup commands on reflex because most tools trained them to. The overwrite
is silent exactly where the stakes are highest: the source is the only source of
truth, and the manifest and starter are the two files init is surest it "owns".

## Resolution

Broke the overwrite out of the design: init is create-only per artifact — what
exists is read and reported, never rewritten; there is no `--force`. A second run
changes no bytes anywhere in the tree.
Answered by `init-changes-nothing-twice`.
