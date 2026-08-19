---
kind: functional
origin: intent
satisfied-by: [Planner]
deferred:
---

# The plan cleans up after itself

A wave that closes takes its working files with it: the index it was diffed against and
the declaration files it consumed are deleted in the same successful `plan next`. A
blocked close deletes nothing — the files are what the retry reads. When the plan
completes, `waves/` is empty and removed with it, so a completed plan's folder holds
records and nothing else: the charter, the tasks, `state.json`. `plan reset` keeps
clearing the whole of `waves/`, as it always did.

## System Context

The working files are input, not record — the standing claims already say so: a
declaration is consumed by `plan next` and never read again, and capture reads only the
current wave's index, so a closed wave's files are dead even in an open plan. The record
of what a wave accounted for is the journal, where every link carries its rule, its
proving test and its commit provenance.

Left on disk they were 13M of dead JSON across 69 indexes, every one in a completed
plan, carried by the tip, every diff and every clone. One sweep removed them by hand;
this claim makes the sweep unnecessary, the way `reset` already proves the tree lives
without the files.

## Satisfy

`Planner` (the successful close deletes the closed wave's index and declarations; the
completion removes the empty `waves/`; a blocked close deletes nothing).

- test — a successful wave close deletes that wave's index and declaration files, and the
  next wave's files stand untouched
- test — a blocked close deletes nothing, and the retry closes on the same files
- test — at DONE the plan folder holds no `waves/` at all
- test — `plan reset` still clears `waves/` whole
