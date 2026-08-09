---
name: archi-finish-worktree
description: Close a worktree — land its spec/plan/code unit, push member branches for their PRs, free the worktree once the receiving branch carries the work, and keep its row as the record. Use when a unit of work in an archi worktree is done and must land. A conflicted merge hands off to archi-merge.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-finish-worktree/SKILL.md`. When the act is
> `updated` or `created`, the text you follow is stale. Read that file
> again, follow it, and only then continue. `ok` means continue.

# Archi finish worktree

One worktree, one landing.

## Ground rules

**A worktree lands only whole.** The plan is closed, because the merge
command refuses mid-wave work. A worktree that carries no plan, only the
spec, lands freely.

**The landing runs from the receiving checkout**, never from inside the
worktree.

**Retirement is the job of the command.** Retirement follows the
integration, not the push: a local merge frees the folder at once, and a
sideways landing or a member push frees it later, once the receiving
branch carries the work. Never run `git worktree remove`, and never edit
`worktrees.toml` by hand.

## The landing, in order

1. **Close the work.** In the worktree: the wave loop finished (`archi
   plan next`) or you ran `archi plan close`. `archi check` is green.
   Everything is committed.
2. **Check before you merge.** From the receiving checkout, run `git log
   --oneline <slug-branch>..<receiving-branch>`. An empty result means
   the receiving branch never moved after the worktree forked, so land
   directly. When it moved, ask the same range about the archive: `git
   log --oneline <slug-branch>..<receiving-branch> -- archi/versions/`.
   - The archive is untouched. This is plain divergence, so merge. When
     an ordinary content conflict appears, take it to the `archi-merge`
     triage.
   - The archive is touched *and* the worktree minted saves of its own.
     The version-id collision is then guaranteed. Do not attempt the
     direct landing. Merge the receiving branch **into the worktree**
     first. Repair the archive there: keep the first-landed archive, run
     `archi version remint -m <note> [--session <slug>]`, and get `archi
     check` green. Commit the printed unit, then land.
3. **Land.** From the receiving checkout, run `archi worktree merge
   <slug> [--to [<member>=]<branch>]...`.
   - Member branches push to their remotes. Their integration is a PR on
     the forge, never a local merge into a member checkout, so the member
     worktree stays until its base carries the work. Open the PR. The
     next archi command frees the folder.
   - The spec merges into the current branch. It lands sideways with
     `--to <branch>` when the receiving branch is protected. A protected
     branch never receives a local merge, so the worktree stays: push the
     landed branch, open the PR, and the folder frees itself on the next
     archi command once the receiving branch carries the work.
   - A local merge puts the work in the receiving branch at that moment,
     so it removes the worktree and closes its row in the same move.
4. **On a refusal, repair and run it again.** The command is idempotent.
   - *open plan* — close it, as in step 1.
   - *protected receiving branch* — use `--to <branch>`, push, and open a
     PR.
   - *stale member baseline* — the merge names each member whose worktree
     tip is past the recorded baseline. Run `archi version anchor --repo
     <member>` in the worktree, then run the merge again.
   - *member push refused*, for no remote or no rights — repair the
     remote and run the merge again. The member stays bound until its
     push lands.
   - *merge conflict* — the `archi-merge` skill handles it. The remint
     runs in this worktree. Re-attach the worktree with `archi worktree
     mint <slug>` when the landing already retired it. Then run the merge
     again to finish the retire.
5. **To abandon instead of landing**, run `archi worktree close <slug>`.
   The folders go away, members included, and the row stays as the record
   of what this machine carried. Unpushed branches stay, for deletion by
   hand.

Where `gh` is available, you may ask the forge whether the PR merged and
then run `archi worktree close <slug>` — optional, never a requirement.

## Failure modes

- `plan <X> is draft/started` — the worktree is mid-work. Only closed
  units land.
- "`main` is protected — never receives a local merge" — this is by
  design. Land with `--to` and a PR.
- "branch `land/x` already exists" — a stale `--to` from an earlier
  failed landing. Delete it and run the landing again. Its work either
  landed already, or it comes again.
- The registry still lists a retired path. It self-heals on the next
  read: the row closes, it does not vanish, because the row is the
  record. A row that still reads as live work is a worktree that git
  still knows.
- A seat still stands after its pull request merged. The sweep compares
  against the *local* receiving branch, and archi never fetches, so a
  merge on the forge stays invisible here. Run `git pull` in the
  receiving checkout. Any archi command then frees the folder and closes
  the row.

The two-writer merge that this skill hands off to is the `archi-merge`
skill. The opening counterpart is the worktree protocol in the `archi`
skill.
