---
name: archi-finish-worktree
description: Close a worktree seat — land its spec/plan/code unit, push member branches for their PRs, retire the worktree and its registry binding in one move. Use when a unit of work in an archi worktree is done and must land; a conflicted join hands off to archi-merge.
---

> **Skill freshness — the first move.** In an initialized project run
> `archi sync-skills` before anything else. If it reports
> `.claude/skills/archi-finish-worktree/SKILL.md` as `updated` (or `created`), the text
> you are following is stale: re-read that file, follow it, and only
> then continue. `ok` means proceed.

# Archi finish worktree

One seat, one landing.

Ground rules, always:

- A seat lands only whole: the plan is closed — the merge verb refuses
  mid-wave work. Spec-only seats (no plan) land freely.
- The landing runs from the receiving checkout, never from inside the seat.
- Retirement is the verb's job: the worktree and its registry entry vanish
  with a clean landing — never `git worktree remove` or edit
  `worktrees.toml` by hand.

## The landing, in order

1. **Close the work.** In the seat: the wave loop finished (`archi plan
   next`) or `archi plan close`; `archi check` green; everything committed.
2. **Pre-flight the join.** From the receiving checkout, before merging:
   `git log --oneline <slug-branch>..<receiving-branch>` — empty means
   the receiving branch never moved since the seat forked: land directly.
   If it moved, ask the same range about the archive:
   `git log --oneline <slug-branch>..<receiving-branch> -- archi/versions/`
   - archive untouched → plain divergence: merge; an ordinary content
     conflict, if one appears, goes to the `archi-merge` triage;
   - archive touched *and* the seat minted saves of its own → the
     version-id collision is guaranteed — do not attempt the direct
     landing. Merge the receiving branch **into the seat** first, run the
     collision ceremony there (keep the first-landed archive,
     `archi version remint -m <note> [--session <slug>]`, `archi check`
     green), commit the printed unit, then land.
3. **Land.** From the receiving checkout:
   `archi worktree merge <slug> [--to [<member>=]<branch>]...`
   - member branches push to their remotes and retire — their integration
     is PRs on the forge, never a local merge into a member checkout;
   - the spec merges into the current branch, or lands sideways with
     `--to <branch>` when the receiving branch is protected — a protected
     branch never receives a local merge: push the landed branch, open a
     PR;
   - a clean landing removes the worktree and clears its binding in the
     same move.
4. **On refusal, repair and re-run** — the verb is idempotent:
   - *open plan* → close it (step 1);
   - *protected receiving branch* → `--to <branch>`, push, PR;
   - *member push refused* (no remote, no rights) → repair the remote and
     re-run; the member stays bound until its push lands;
   - *merge conflict* → the join is the `archi-merge` skill's ceremony;
     the remint runs in this seat — re-attach it with
     `archi worktree mint <slug>` if the landing already retired it — then
     re-run the merge to finish the retire.
5. **Abandoning instead of landing** is `archi worktree drop <slug>`:
   worktrees go, unpushed branches stay for hand deletion.

## Failure modes

- `plan <X> is draft/started` → the seat is mid-work; only closed units
  land.
- "`main` is protected — never receives a local merge" → by design; land
  with `--to` and a PR.
- "branch `land/x` already exists" → a stale `--to` from an earlier failed
  landing: delete it (its work either landed already or is coming again)
  and re-run.
- The registry still lists a retired path → it self-heals on the next
  read; a row that survives is a live worktree git still knows.

The two-writer join this hands off to is the `archi-merge` skill; the
opening counterpart is the seat protocol in the `archi` skill.
