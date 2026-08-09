---
kind: functional
origin: stressor(ignored-build-junk-vetoes-every-cleanup)
satisfied-by: [Seats.Landing]
deferred:
---

# ignored files never veto a cleanup

The sweep reads the tree as git reports it: modified tracked files and
untracked files that no ignore rule covers. When that reading is clean,
the folder is removed forcibly, whatever ignored build output it holds.
Ignored files are reproducible by definition, and they are the reason
the folder is worth freeing.

## System Context

`git worktree remove` refuses on any untracked file, so a plain removal
never succeeds on a real project: `node_modules`, `target` and build
caches are exactly what makes a seat weigh tens of gigabytes.

## Satisfy

`Seats.Landing` decides on `git status --porcelain`, which is silent
about ignored files, and then removes with force.

- test — a seat holding an ignored build directory is freed once its
  work is integrated
- test — a seat holding an untracked file that no rule ignores is left
  alone
