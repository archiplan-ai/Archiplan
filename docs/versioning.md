# Versioning

A version is a semantic snapshot of the model — the unit of *we agreed it looked like this*:
something to stress against, plan from and diff between. It is durable in a shallow clone,
immune to git rewrites, and minted only when meaning moves. The live tree holds exactly one
model; saved versions are archived forms of its canonical render, reconstructable bit for bit
and cheap enough to keep forever.

The whole archive works from the files alone. Git history is provenance and a recovery path,
never a dependency.

## What a version is

The compiled model renders to a single canonical `.arch` source. A version *is* that render,
identified by the sha256 of its bytes. Three consequences:

- **Identity is the render alone.** Notes, timestamps, commit provenance and code baselines are
  metadata recorded beside a version, never part of what makes it that version — the same model
  saved in two projects gets the same hash.
- **Minted only on change.** `version save` compares the live render's hash to the latest entry;
  equal means nothing to mint. A save never produces a duplicate.
- **"Current" is derived, never stored.** `version current` hashes the live render on demand and
  looks for a match, so it cannot drift out of sync with the truth: it reports *at `<id>`*, or
  *dirty since `<id>`* when the working tree carries unsaved changes.

## Commands

| Command | Does |
|---|---|
| `version save -m <note>` | Compile the live tree and mint a version if the render moved. The note is mandatory. |
| `version current` | Which saved version the live render matches, or *dirty since* the latest. |
| `version list` | Every version: id, timestamp, encoding, note. |
| `version show <id>` | Materialize a version's canonical source — compilable, so it can seed a tree or ground a read. |
| `version diff <a> <b>` | The semantic delta between two versions. Either side may be `live` to compare the working tree. |
| `version anchor [--repo <m>]` | Record commit provenance (or a member baseline) on the version the live render matches — post hoc. |
| `version remint -m <note> [--session <slug>]` | Merge path: mint a merged tree onto the lineage and re-stamp a round. See *Parallel edits*. |

Every command also takes `--project <dir>` to point at a project other than the working directory.

```bash
archi version save -m "split the gateway from the router"
# saved v0007 (patch, 812 bytes, archi/versions/v0007.arch.patch) — split the gateway from the router
# commit as one: archi/versions/index.toml, archi/versions/v0007.arch.patch

archi version current            # at v0007
archi version diff v0006 v0007   # what that save changed
archi version diff v0007 live    # unsaved changes since the last save
```

## The archive on disk

```
archi/versions/
  index.toml          append-only manifest; the hashes it records seal everything else
  v0001.arch          keyframe — a whole canonical render
  v0002.arch.patch    patch — a unified diff from the previous version
  v0003.arch.patch
  ...
  .gitattributes      marks v*.arch generated, so forges collapse keyframes in review
```

Ids are a dense sequence (`v0001`, `v0002`, …) and each entry names its parent, so the archive
is one linear chain.

**Keyframes and patches.** The first version is a keyframe. A later save writes a keyframe
exactly when the patches since the last keyframe — this one included — together outgrow the new
render; otherwise it writes a unified diff against the previous version's bytes. Total archive
size therefore stays within about twice the keyframe, whatever the churn. Patches are the
permanent, reviewable change record of each round; keyframes are generated artifacts.

**Sealed.** Each entry records the sha256 of its render. Reconstruction walks from the nearest
keyframe at or before the target, applies the forward patches, and verifies the result against
the seal. Editing any keyframe, patch or manifest entry breaks it. `archi check` re-verifies the
whole chain — dense ids, a linear parent chain, an opening keyframe, every seal, and no files the
manifest doesn't name — and a break is a compile error, not a lint finding.

## Provenance

On a **clean** project tree, `save` records the current commit on the entry: that commit really
contains the sources the render came from. A dirty tree records no commit — the render is sealed
all the same, only its git provenance waits.

`version anchor` records it after the fact: commit the tree, then anchor while its render still
matches the version. Provenance is a birth fact — once recorded, anchor reports it and never
rewrites it, even after HEAD has moved on. Nothing in the archive depends on the commit; it is
there for humans and recovery.

## Baselines (multi-repo)

When the project declares `[[repo]]` members (see [multi-repo-workflow.md](multi-repo-workflow.md)),
a save also records where each member's code stood — a *baseline* — but only for members whose
tree is clean. Omissions are named at save with their recovery, never silent:

```
baseline backend: a1b2c3d
no baseline for `web`: its tree is dirty — commit it, then `archi version anchor --repo web`
```

`archi version anchor --repo <member>` records a member baseline post hoc. It is marked
*anchor-born*, and reports word the span between the save and the anchor as unaudited — the
save-time recording is the stronger guarantee. A memberless project carries no baseline machinery
at all; its entries look exactly as they did before the first member was declared.

## In the loop

`save` is the one mint point in the archi loop. Besides sealing the model it closes the open
stress round against the new version, fires its incidence report, and prints the set of files that
must land as a single commit — the manifest, the new keyframe or patch, and the round's session
stamp.

**Parallel edits.** Two branches that each saved will have minted the same id, and `index.toml`
collides on merge with conflict markers. Keep the first-landed entry and its file — both sides'
model and doc work is already merged — then carry the later round onto the lineage with
`archi version remint -m <note> --session <slug>`, which mints the merged tree as the next id and
re-stamps that round's `closed:` to it.

## Failure modes

- `version save` says *nothing to mint / unchanged since `<id>`* → the render didn't move; there
  is nothing to save. Any open round still closed against `<id>`.
- `version current` says *dirty since `<id>`* → the working tree has unsaved semantic changes.
  `save` them, or `diff <id> live` to see what moved.
- `archi check` reports a broken seal → a keyframe, patch or manifest entry was hand-edited.
  Restore `archi/versions/` from git history; the archive is append-only through `save`.
- `version anchor` says *matches no saved version* → the live render is dirty. `save` first,
  commit, then anchor.
- `index.toml` has conflict markers → two branches minted the same id; resolve with
  `version remint` (see *Parallel edits*).
- save says *no baseline for `m`* → that member's tree was dirty or unreachable. Commit it, then
  `archi version anchor --repo m`.
