---
name: archi-merge
description: Merge two branches that both mutated an archiplan spec — triage the merge with check, resolve version-archive collisions with remint, read the notes the journal absorbed, fold concurrent stress rounds. Use when you integrate parallel spec work or repair a broken state after a merge.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-merge/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

# Archi merge

Two writers changed one spec.

## Ground rules

**The contract is the canonical render, not the text.** A clean git merge
proves nothing. Two green branches prove nothing. The *composition* is
what breaks, and it breaks for the person who integrates it. Run `archi
check` on every merge commit.

**Content files resolve as ordinary text** — the model, the requirements
and the stressors. Repair lifecycle state only through the commands. That
state is the archive, the journal and the `closed:` stamps, and the
recipes below never hand-edit it.

**Whoever merges authored neither side.** When a diagnostic names a
recipe, follow the recipe. When it names the work of the other writer,
ask them.

## The merge, in order

1. **Merge.** Conflicts, when they appear, land in content files and in
   at most two archive files. Resolve the content files normally, and
   repair the archive as the next section describes. The journal does not
   conflict, because `archi/links/.gitattributes` union-merges it. A
   landing that stopped on conflicts arrives here mid-merge. That landing
   is `archi worktree merge`, the archi-finish-worktree skill. The triage
   is the same, and after the merge a re-run of the landing finishes the
   retire.
2. **Triage with `archi check`.**
   - *Green* — go to step 3.
   - *`E_UNKNOWN_NAME`, `E_MODEL_REF` or `E_SLUG`* — a semantic conflict
     that is textually clean. One branch deleted or renamed what the
     other now references, or two branches claimed one slug. The
     diagnostics are located. Fix them as ordinary spec edits.
   - *`E_ARCHIVE: … holds merge conflict markers`* — both branches minted
     the same version id. Repair the archive as the next section
     describes. Session id-validation stays quiet while the archive is
     unreadable. That silence is by design. It is not more breakage.
   - *`E_SESSION`* — two rounds are both open, or a marker fused a round.
     Use the fold recipes below.
3. **Review what the merge composed.** `archi version diff <latest> live`
   renders the merged working tree canonical and diffs it against the
   archive. This is the semantic delta that neither author saw, and you
   can read it *before* a save seals it. A merge that compiles green can
   still move meaning under a standing claim. This diff is where you
   catch that.
4. **Read what the journal absorbed.** Run `archi link verify`. Every
   `journal:` note is one writer's op absorbed on a record the other
   writer had already retired: a repin of a link the other branch
   retired, or a double retire. The removal wins, and the note is the
   surface of the disagreement. When the link must live, run `link add`
   to add it fresh. Never resurrect a link by editing the journal.

## Repair the archive: both branches minted vNNNN

The conflict is contained by design. It touches
`archi/versions/index.toml` plus one `vNNNN.arch.patch`, or one `.arch`
for a small model that keyframes. The model, doc and session work of both
writers is already merged. Only the archive needs the repair.

**Run the repair in the worktree of the later round, never in the primary
checkout.** The primary checkout refuses mutation, and that refusal is
correct: the repair is the work of the later writer. When that worktree
is already retired, because `--to` landed it, re-attach it first. `archi
worktree mint <slug>` picks the surviving branch back up. Then merge the
receiving branch *into* the worktree, run the repair there, and land
again. Delete a stale `--to` branch from the failed landing, or choose a
fresh name, before you land again.

1. **Keep the whole first-landed archive.** When you merge main into your
   branch, run `git checkout --theirs archi/versions/`. When you merge
   your branch into main, use `--ours`. Commit the merge.
2. **Review your delta on top of theirs:** `archi version diff
   <winner-id> live`.
3. **Re-mint your round onto the lineage:**

   ```
   archi version remint -m "<your original note>" --session <your-round-slug>
   ```

   One command has three effects. It mints the merged tree as the next id.
   It re-stamps the `closed:` of your session onto that id, because the
   round record follows its answers. It prints the `commit as one:` unit.
   Your original note is on your branch: `git show
   <your-branch>:archi/versions/index.toml`.
4. **Commit exactly the printed unit.** The manifest, the patch or
   keyframe, and the session stamp travel as one commit.

Remint refuses three states, and each one means the repair does not
apply. An unchanged model carries nothing, because the merge brought no
delta of yours. An unknown session slug is a typo. An open session closes
through `version save`, and remint re-stamps only a round whose closing
save the merge discarded. Without `--session` it mints, and it re-stamps
nothing.

## Fold concurrent rounds

Sessions merge as ordinary files, so a merge can assemble a round that
nobody authored. Never resolve this by hand under `archi/stress/`.
Markers there are one `E_SESSION` that names its recipe, and the fold
command is the only path that merges round records.

- **Same slug, marker-fused file** — `archi session fold <slug> -m <note>
  [--keep theirs]`. The charter of the kept side survives. The other
  charter lands under `## Folded:` with the label of its merge side.
- **Two rounds both open** — `archi session fold <loser> --into <winner>
  -m <note>`. The stressor files move, and a name collision refuses,
  because a stressor is one writer's pressure. The charter, the pin and
  the stamp of the loser land under `## Folded: <loser>`, and the folder
  of the loser is deleted.
- **A fused sealed pair** folds with the surviving stamp intact and the
  folded stamp at `pending remint`. That is a finding until `archi
  version remint -m <note> --session <slug>` re-stamps it. The order is
  archive, then fold, then remint. Each step's diagnostic names the next
  one.
- A fold refuses across different pins, and across mixed open and sealed
  pairs. Split those by hand.

The stressor files themselves union cleanly. One pressure per file exists
exactly so that parallel stress work lands as parallel files.

## Failure modes

- `E_ARCHIVE: cannot read vNNNN… travel as one commit` — a half-shipped
  save. The author committed the manifest without the untracked patch,
  the classic `commit -am`. Recover the file from the tree of the author
  or from the history of their branch. The fix is theirs to push.
- Conflict markers inside `archi/links/journal.jsonl` — the union
  attribute was missing at merge time. Resolve it by keeping every event
  line from both sides, and drop only the marker lines. Suffixed ids
  cannot collide, and the fold absorbs the leftovers. Then make sure
  `archi/links/.gitattributes` is present. Any journal append self-heals
  it.
- `journal corrupt: … never minted` — this is real corruption, not a
  merge leftover. A lost add or a bad paste causes it. Recover the
  journal from the git history. Do not repair the lines by hand.
- `E_SLUG` after a clean merge — the doc namespace is project-wide and
  branch-blind. Rename one side. Name early and integrate often.
- The `closed:` of a session names a version that is not the round's own
  — its closing save lost the collision, and nobody reminted. Repair the
  archive as above. The re-stamp is the job of remint, never a hand edit.

The loop this skill plugs into is the `archi` skill.
