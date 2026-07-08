---
name: archi-merge
description: Merge two branches that both mutated an archiplan spec — triage the join with check, resolve version-archive collisions with remint, read the journal's absorbed residue, fold concurrent stress rounds. Use when integrating parallel spec work or repairing a post-merge broken state.
---

# Archi merge

Two writers, one lineage. Ground truth: `requirements/multiplayer.md`; the
failure inventory is the `merge-pressure` round (`archi/stress/merge-pressure/`).

Ground rules, always:

- The contract is the canonical render, not the text. Git merging clean
  proves nothing; both branches green proves nothing — the *composition* is
  what breaks, and it breaks at the integrator's desk. Run `archi check` on
  every merge commit.
- Content files (model, requirements, stressors) resolve as ordinary text.
  Lifecycle state (archive, journal, `closed:` stamps) is repaired only
  through verbs — the recipes below never hand-edit it.
- Whoever merges authored neither side. When a diagnostic names a recipe,
  follow the recipe; when it names the other writer's work, ask them.

## The join, in order

1. **Merge.** Conflicts, if any, land in content files (resolve normally)
   and at most two archive files (the collision ceremony below). The
   journal does not conflict — it union-merges via
   `archi/links/.gitattributes`.
2. **Triage with `archi check`.**
   - *Green* → go to 3.
   - *`E_UNKNOWN_NAME` / `E_MODEL_REF` / `E_SLUG`* → a textually-clean
     semantic conflict: one branch deleted or renamed what the other now
     references, or two branches claimed one slug. Located diagnostics;
     fix as ordinary spec edits.
   - *`E_ARCHIVE: … holds merge conflict markers`* → both branches minted
     the same version id: the collision ceremony (next section). Session
     id-validation stays quiet while the archive is unreadable — that
     silence is by design, not more breakage.
   - *`E_SESSION: … both open`* → two concurrent rounds: fold by hand
     (below).
3. **Review what the merge composed.** `archi version diff <latest> live`
   renders the merged working tree canonical and diffs it against the
   archive — the semantic delta neither author saw, readable *before* any
   save seals it. A merge that compiles green can still have moved meaning
   under a standing claim; this diff is where you catch it.
4. **Read the journal's residue.** `archi link verify` — any `journal:`
   note is one writer's op absorbed on the other's tombstone (a repin of a
   link the other branch retired, a double retire). The subtraction wins;
   the note is the surface of the disagreement. If the link should live,
   `link add` it fresh; never resurrect by editing the journal.

## The collision ceremony: both branches minted vNNNN

The conflict is contained by design: `archi/versions/index.toml` plus one
`vNNNN.arch.patch` (or `.arch` — small models keyframe). Both writers'
model, doc and session work is already merged; only the archive needs the
ceremony.

1. **Keep the first-landed archive wholesale.** Merging main into your
   branch: `git checkout --theirs archi/versions/`. Merging your branch
   into main: `--ours`. Commit the merge.
2. **Review your delta on top of theirs:** `archi version diff <winner-id> live`.
3. **Re-mint your round onto the lineage:**

   ```
   archi version remint -m "<your original note>" --session <your-round-slug>
   ```

   One verb, three effects: mints the merged tree as the next id, re-stamps
   your session's `closed:` onto it (the round record follows its answers),
   and prints the `commit as one:` unit. Your original note is on your
   branch: `git show <your-branch>:archi/versions/index.toml`.
4. **Commit exactly the printed unit** — manifest, patch/keyframe, session
   stamp travel as one commit.

Remint refuses three states, each meaning the ceremony does not apply:
an unchanged model (nothing to carry — the merge brought no delta of
yours), an unknown session slug, and an open session (an open round closes
through `version save`; remint re-stamps a round whose closing save the
merge discarded). Without `--session` it mints and re-stamps nothing.

## Folding concurrent rounds (the one by-hand join)

`rounds-fold-deliberately` is still open: sessions merge detectably but
fold manually. Until the verb lands:

- **Different names, both open** (`E_SESSION` names both): pick the
  surviving session; `git mv` the other's stressor files into its folder;
  merge the two charters' prose into the survivor — both "why"s survive,
  in words, not by picking a side; delete the other session's anchor file;
  re-run `check`.
- **Same name on both branches** (add/add conflict on the anchor file):
  the conflict is your only warning — resolve by merging both charters by
  hand. Taking one side silently adopts the other writer's stressors under
  a charter they never wrote (the chimera round).
- Stressor files themselves union cleanly — one pressure per file exists
  exactly so parallel stress work lands as parallel files.
- Team discipline until then: one *open* session at a time across
  branches; closing rounds in parallel is safe (that collision has the
  ceremony above).

## Failure modes

- `E_ARCHIVE: cannot read vNNNN… travel as one commit` → a half-shipped
  save: the author committed the manifest without the untracked patch
  (classic `commit -am`). Recover the file from the author's tree or their
  branch history; the fix is theirs to push.
- conflict markers inside `archi/links/journal.jsonl` → the union
  attribute was missing at merge time. Resolve by keeping every event line
  from both sides (drop only the marker lines) — suffixed ids cannot
  collide and the fold absorbs the residue — then make sure
  `archi/links/.gitattributes` is present (any journal append self-heals
  it).
- `journal corrupt: … never minted` → not merge residue but real
  corruption (a lost add, a bad paste). Recover the journal from git
  history; do not hand-repair lines.
- `E_SLUG` after a clean merge → the doc namespace is project-wide and
  branch-blind; rename one side. Name early, integrate often.
- A session's `closed:` names a version that isn't the round's own → its
  closing save lost the collision and nobody reminted; run the ceremony —
  the re-stamp is remint's job, never a hand edit.

Depth: `requirements/multiplayer.md`, `requirements/versioning.md`
(Versioning * Multiplayer), `skills/archi.md` (the loop this plugs into).
