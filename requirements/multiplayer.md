# Multiplayer

Multiple writers mutate the spec on parallel git branches and merge. Git is the transport and
the only coordination layer — no server, no locks, no live session. The discipline is therefore
a **merge contract per store**: every store in the tree must either merge cleanly under git's
text semantics, or detect its broken post-merge state, name the recipe in the diagnostic, and
offer a verb that repairs it. No repair may require hand-editing a file the lifecycle rules
forbid editing.

Ground truth for this document: the `merge-pressure` stress round
(`archi/stress/merge-pressure/`), which ran every case below against the binary — two clones
off a shared origin, real conflicts, real resolutions.

## The stores and their joins

**Model text (`src/*.arch`).** Git merges it, but the contract is the canonical render, not the
text ([versioning.md](versioning.md)). Textually disjoint edits compose into unreviewed
semantics: broken (an edge whose node the other branch deleted — loud, but only at the
integrator's post-merge check) or silently drifted (a retyped carrier under a standing
requirement claim — green everywhere). The join needs a semantic review surface before the
seal: the live render diffed against any archived version (`merge-deltas-are-reviewable`),
runnable in CI on merge commits.

**Version archive (`archi/versions/`).** Ids are a dense sequence minted from a branch-local
view, so parallel saves collide in the manifest and the patch filename — loudly, in exactly two
files, while everything else about both rounds merges clean. The collision is the design
working; the repair is not yet designed. The later writer re-mints onto the merged lineage
(`remint-rejoins-the-lineage`): the post-merge state detected and named with its recipe, the
note carried over from the discarded entry, the round's `closed:` stamp moved to the reminted
id. A save's artifacts — manifest entry, patch or keyframe, session stamps — travel as one
commit, and check names a half-shipped save at its author rather than as a read error at every
clone.

**Link journal (`archi/links/journal.jsonl`).** Append-only, sequential-replay truth — and any
two branch-parallel link ops conflict at its tail, forcing by-hand resolution of the one file
hand edits are banned from. The fold must accept concurrent histories
(`the-fold-survives-a-merge`): a union in either order reaches one defined live set, parallel
id mints don't collide (or a verb re-sequences them), events landing on tombstones are defined
rather than corruption, and repairs are verbs that leave journal records. Until then the
journal is single-writer per repository.

**Stress sessions (`archi/stress/`).** "At most one open session" is a repository invariant
that parallel branches violate silently — git sees only unrelated folders. And same-name
sessions can chimera: one add/add conflict, a keep-a-charter resolution, and the merged repo
holds a green round nobody authored, closed and incidence-reported as if designed. Folding two
rounds into one is legitimate and must be deliberate: detected, recipe-named, both charters
preserved in the surviving record (`rounds-fold-deliberately`).

**Slugs.** The doc namespace is project-wide and branch-blind; two branches can each be green
and collide at the join (requirement vs session of the same name). Detection is good —
`E_SLUG` names both files — but only at the post-merge check. Discipline: name early, integrate
often, keep `archi check` on merge commits.

## The integrator's position

Whoever merges authored neither side, and today inherits every consequence: the compile break
neither branch could see, the session worklist of both writers, the journal conflict with no
legal resolution, and diagnostics that cascade (one manifest conflict reads as a corrupt
archive plus an `E_SESSION` for every session in the repo). Every post-merge broken state must
therefore be (a) detected by `archi check`, (b) named together with its repair recipe, and
(c) repairable by verbs alone. Today (a) mostly holds; (b) and (c) mostly do not. The gap is
tracked in `issues/multiplayer-is-a-stub.md` and carried by the four derived requirements of
the merge-pressure round.

## Non-goals

Real-time co-editing, locks, server arbitration, CRDT stores. Git branches and merges are the
concurrency model; the tool's job is to make the join safe, not to prevent the fork.
