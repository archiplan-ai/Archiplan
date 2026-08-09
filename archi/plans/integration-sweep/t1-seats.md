---
node: Seats
owns: [a-seat-lives-until-its-work-lands, nothing-leaves-the-registry, integration-is-proven-by-content, a-resumed-seat-is-live-again, ignored-files-never-veto-a-cleanup, only-an-active-row-binds, merge-retires-the-worktree, member-branches-land-by-push]
---

# t1 — Seats

The registry row gains `status: active | closed` (serde default active,
every existing file reads) and an optional landing record per side:
`{ branch, receiving, sha }` for the spec, the same three for each
member (receiving = the member's recorded base). The landing writes the
record instead of retiring on the sideways `--to` path and on every
member push; the local-merge path still retires at once and now closes
the row instead of deleting it. Self-heal closes rows git no longer
backs. Every licensing lookup reads active rows only.

The integration probe is one shared helper both the landing and the
sweep call: `merge-base --is-ancestor` first, an empty tree diff second
(the squash answer), both read-only. A landing record stops counting
when the seat's head moved past its sha or the tree is unclean —
derived on read, never written. The sweep frees a folder only when the
probe passes and `git status --porcelain` is silent (ignored junk never
vetoes, so the removal is forced), then marks that side done; when the
spec and every member are done the row closes. Members sharing one
physical repository share one folder and one verdict. An unreachable
member yields no verdict and no action.

## Spec

- `Seats`
- `Seats.Registry`
- `Seats.Landing`
- `Service type_of Seats`
- `Storage type_of Seats.Registry`
- `Cli.drive consult(->Command, <-Report) Seats.bind`
- `Cli.drive consult(->Command, <-Report) Seats.guard`
- `Cli.drive consult(->Command, <-Report) Seats.land`
- `Cli.drive consult(->Command, <-Report) Seats.mint`
- `Cli.drive consult(->Command, <-Report) Seats.survey`
- `Cli.drive consult(->Command, <-Report) Seats.verdict`

## Inputs

## Outputs

- crates/archi/src/worktrees.rs

## Stack

- serde default on the status and landing fields; the registry TOML keeps deny_unknown_fields
- git merge-base --is-ancestor and git diff --quiet through the gitcmd plumbing
- git status --porcelain for the safety read; worktree_remove with force after it

## Verifications

### a-seat-lives-until-its-work-lands

- test — worktree_e2e: a `--to` landing keeps the worktree and the row and records the branch; a member push keeps its member worktree; a local merge retires in the same move and closes the row

### nothing-leaves-the-registry

- test — worktree_e2e: close marks the row and keeps it; a hand-removed worktree closes its row on the next read; a mint of a closed slug re-opens the same row

### integration-is-proven-by-content

- test — worktree_e2e: a fast-forward merge proves integrated by ancestry; a squashed equivalent commit proves integrated by content; an unmerged branch proves neither

### a-resumed-seat-is-live-again

- test — worktree_e2e: a commit on top of a landed seat, and separately an uncommitted change, each make the sweep skip the folder and the row read as live

### ignored-files-never-veto-a-cleanup

- test — worktree_e2e: an integrated seat holding an ignored build directory is freed; one holding an unignored untracked file is left alone

### only-an-active-row-binds

- test — worktree_e2e: a closed row leaves its checkout unbound and does not own its plan

### merge-retires-the-worktree

- test — worktree_e2e: the local merge path keeps its whole contract — merged, branch deleted, worktree gone — and the row is closed rather than deleted

### member-branches-land-by-push

- test — worktree_e2e: a pushed member keeps its worktree until its base carries the work, and a refused push keeps the member bound as before
