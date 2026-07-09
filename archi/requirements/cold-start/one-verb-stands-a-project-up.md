---
kind: functional
origin: intent
satisfied-by: [Scaffold, Cli]
deferred:
---

# One verb stands a project up

`archi init [<dir>]` turns a directory (default the working one) into an archiplan
project: the source dir with a commented starter module, the agent briefing
installed, and — written last, so a project root never fronts a half-made tree —
`archi.toml` with the directory's name and the default preset. `archi build` passes
on the result, compiling zero statements; the report names every artifact with what
happened to it; exit 0 on success and on the nothing-to-do re-run, 1 when the tree
refuses a write, 2 on a malformed invocation.

## System Context

Every other verb locates an existing project and is a usage error without one —
init is the verb before that precedence exists, so it takes its target as a
positional argument instead of `--project`. Downstream of it: `build` and `check`
compile immediately, the greenfield skill's step one stops being hand work, and an
interrupted or repeated run is the ordinary case, not the exception
(`init-changes-nothing-twice`).

## Satisfy

`Scaffold` (reads what the target already holds, emits the missing artifacts in an
order that keeps every intermediate tree honest). `Cli` (the `init` verb: target
argument, report rendering, exit codes).

- test — e2e: `archi init <dir>` then `archi build --project <dir>` passes, compiling zero statements
- test — the fresh report names each artifact created, the manifest on its last created line
- test — a second positional argument is a usage error, exit 2
