# Multiplayer: the discipline is specified, the verbs are missing

**Kind:** missing feature · opened by the bootstrap's first round (`parallel-rounds`), mapped
store by store by the `merge-pressure` round (round 9, 2026-07-08)

**Status:** resolved 2026-07-08 — all four joins landed. Three via plan `make-the-join-safe`
(fold survives merges + collision-free link ids + union attribute; `version remint --session`
with recipe-naming diagnostics and the commit-as-one unit; `version diff <id> live`), the
fourth via the `fold-pressure` round and plan `fold-on-purpose` (`archi session fold`,
marker-reading `E_SESSION` detection, the `## Folded:` trace, `pending remint` re-stamping) —
verified by `multiplayer_e2e.rs`, `session_e2e.rs` and two-clone lab replays of both rounds'
scenarios. The spec-side execute loop under merge (parallel plans, capture racing capture)
remains a future round, tracked by the merge-pressure charter's stated non-scope.

`requirements/multiplayer.md` now specifies the per-store merge contract. The merge-pressure
round pressed every store through real branch-and-merge cycles; what the binary does today at
each join:

- **Archive** — collision is loud and contained (2 files) ✓; but mid-conflict `check` cascades
  (one collision → `E_ARCHIVE` + `E_SESSION` for every session in the repo, no recipe named);
  the instinctive union of the manifest is TOML garbage (`duplicate key`), so the dense-id
  guard never speaks; the remint works but retypes its note from memory, closes no session, and
  leaves the later writer's session `closed:` naming the *winner's* version — green check,
  lying record, repairable only by a forbidden hand edit. Separately, `commit -a` ships half a
  save (manifest without the untracked patch): the author stays green forever, every clone gets
  `E_ARCHIVE: cannot read …` with no hint of the cause.
- **Model text** — structural breaks (`E_UNKNOWN_NAME`) and slug collisions (`E_SLUG`) detect
  correctly and are well-located, but only post-merge at the integrator; non-structural drift
  (a retype under a standing claim) is fully silent; no live-vs-archive semantic diff exists —
  the first review surface is the next save's patch, after the seal.
- **Journal** — every branch-parallel pair of link ops is a tail conflict in the
  hand-edit-banned file; a union with duplicate ids folds to `journal corrupt: added twice` and
  every link verb exits 1; retire∥repin folds by line order into either silent judgment loss or
  `journal corrupt: repin names … not a live link`; no renumber, re-sequence, or fold verb
  exists.
- **Sessions** — two open sessions merge silently (only the post-merge check raises
  `E_SESSION`, with a good message); same-name sessions chimera into a round nobody authored
  that closes green with a merged incidence record; folding is possible only as untracked hand
  file-ops that delete a charter.

## Fix shape

The four derived requirements of merge-pressure, roughly in dependency order:

1. ✅ `the-fold-survives-a-merge` — landed: content-suffixed ids (`l0428-ecef31`) cannot collide
   across branches; `archi/links/.gitattributes` (self-healed on first append) union-merges the
   journal without conflict markers; the fold absorbs identical replays and tombstone-landing
   events and surfaces them through `link verify`/`link audit`; only never-minted ids and
   two-links-one-id remain corruption. Note preserved from the round: `remint` takes `-m`, the
   discarded note stays retrievable from the conflicted manifest in git history.
2. ✅ `remint-rejoins-the-lineage` — landed: conflict markers in the manifest raise one
   `E_ARCHIVE` naming the state and the recipe (session validation stays quiet while the archive
   is unreadable); `archi version remint -m <note> --session <slug>` mints the merged tree,
   refuses unchanged models and open or unknown sessions, and re-stamps the round's `closed:`;
   every mint prints its `commit as one:` artifact unit; a manifest entry whose file is missing
   names the half-shipped save.
3. ✅ `merge-deltas-are-reviewable` — landed: `archi version diff <a|live> <b|live>` renders the
   working tree canonical and diffs either direction against any archived version.
4. ✅ `rounds-fold-deliberately` — landed via the fold-pressure round (which first falsified the
   planned mechanism: `merge=union` on session files auto-commits a schema-perfect chimera, so
   the chimera guard reads git's own markers instead of adding identity): markers anywhere under
   `archi/stress/` are one recipe-naming `E_SESSION`; `archi session fold <slug>` normalizes a
   fused file (`--keep theirs`), `fold <loser> --into <winner>` joins two open folders; both
   charters land under a schema-validated `## Folded:` heading with `pin:`/`closed:`/`note:`;
   folds across pins and mixed open/sealed pairs refuse; a fused sealed pair folds `pending
   remint` and `version remint --session` re-stamps the folded stamp, never the surviving one;
   save and remint refuse fused records outright.

## Resolution

The join is verbs all the way down now. Per store: the journal self-merges (union attribute,
absorbing fold), the archive collision names its remint, the model review is `version diff
live`, and the round record folds through `archi session fold` with the trace in the file. The
fold-pressure round's lab falsified the obvious mechanism before it shipped — `merge=union` on
prose auto-commits chimeras — which is why the chimera guard is marker-reading detection rather
than session identity: the boundary git already draws is the one honest signal, and every
merged state now lands as one diagnostic naming the next verb. Integration discipline reduces
to: merge, `archi check`, run what it says, commit as one.
