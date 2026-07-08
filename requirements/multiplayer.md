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
requirement claim — green everywhere). The review surface exists
(`merge-deltas-are-reviewable`, landed): `archi version diff <id> live` renders the working
tree canonical and diffs it against any archived version — the merge's semantic delta is
reviewable, and CI-able, before any save seals it.

**Version archive (`archi/versions/`).** Ids are a dense sequence minted from a branch-local
view, so parallel saves collide in the manifest and the patch filename — loudly, in exactly two
files, while everything else about both rounds merges clean. The collision is the design
working; the repair is a verb (`remint-rejoins-the-lineage`, landed): conflict markers in the
manifest raise one `E_ARCHIVE` that names this exact state and its recipe — keep the
first-landed entry, then `archi version remint -m <note> --session <slug>` — and the remint
mints the merged tree and re-stamps the named round's `closed:` onto the new id, so the record
follows its answers. Every mint prints its artifacts — manifest entry, patch or keyframe,
session stamp — as one `commit as one:` unit, and a manifest entry whose file is missing names
the half-shipped save instead of raising a bare read error.

**Link journal (`archi/links/journal.jsonl`).** Append-only, sequential-replay truth — and now
it folds concurrent histories (`the-fold-survives-a-merge`, landed). Ids carry a content
suffix (`l0428-ecef31`), so parallel mints cannot collide; a `.gitattributes` shipped beside
the journal union-merges branch appends without conflict markers; and the fold absorbs what a
sequential history would forbid — an identical replayed line, an event landing on a
tombstone — surfacing every absorption through `link verify` and `link audit` as notes.
Subtractions still stick, and an event naming an id the journal never minted is still
corruption: tolerance extends exactly as far as merge residue.

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

Whoever merges authored neither side. Every post-merge broken state must therefore be
(a) detected by `archi check`, (b) named together with its repair recipe, and (c) repairable
by verbs alone. For the journal, the archive and the model-review surface all three now hold:
the journal merges itself and its fold surfaces the residue, the manifest collision names the
remint recipe in one diagnostic (session validation stays quiet while the archive is
unreadable), and the live diff reviews the merge before the seal. The remaining gap is the
session record: two open sessions and the same-name chimera are detected but folded only by
hand (`rounds-fold-deliberately`, open — tracked in `issues/multiplayer-is-a-stub.md`).

## Non-goals

Real-time co-editing, locks, server arbitration, CRDT stores. Git branches and merges are the
concurrency model; the tool's job is to make the join safe, not to prevent the fork.
