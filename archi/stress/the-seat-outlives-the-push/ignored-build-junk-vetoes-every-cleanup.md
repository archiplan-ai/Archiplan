---
affects: [Seats.Landing]
outcome: breaking
---

# ignored build junk vetoes every cleanup

The seat holds `node_modules`, `target`, a build cache — thirty
gigabytes of ignored files, which is exactly why the folder is worth
freeing. `git worktree remove` refuses on any untracked file.

## Attractor

The automatic sweep never removes anything on a real project. The
feature exists and frees nothing, and the disk fills with proven-safe
folders.

## Resolution

Ignored files never veto a cleanup: they are reproducible by
definition. The sweep reads the tree as git reports it — modified
tracked files and untracked files that are not ignored — and removes
the folder forcibly when that reading is clean. Derived
`ignored-files-never-veto-a-cleanup`.
