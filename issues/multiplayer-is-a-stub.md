# Multiplayer: the discipline is specified, the verbs are missing

**Kind:** missing feature · opened by the bootstrap's first round (`parallel-rounds`), mapped
store by store by the `merge-pressure` round (round 9, 2026-07-08)

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

1. `the-fold-survives-a-merge` — journal: collision-free ids (or a re-sequencing verb), defined
   tombstone races, order-independent union fold, repair verbs that leave records.
2. `remint-rejoins-the-lineage` — the save collision recipe as a first-class path: detected
   state named by `check` (no cascade), note preserved, `closed:` re-stamped, and a
   save-artifacts-travel-together guard that names a half-shipped save at its author.
3. `merge-deltas-are-reviewable` — semantic diff of the live render against any archived
   version (`version diff` grows a live target), CI-able on merge commits.
4. `rounds-fold-deliberately` — a session fold verb with a record; chimera guard (session
   identity beyond the slug).

Until these land the operating rule stays **one writer per repository**
(`archi/requirements/self-hosting/parallel-editing-discipline.md`).
