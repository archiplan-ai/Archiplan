---
kind: functional
origin: stressor(a-re-run-clobbers-the-tree, a-half-init-strands-the-tree)
satisfied-by: [Scaffold]
deferred:
---

# Init changes nothing twice

Init is create-only per artifact: each one found on disk is read and reported —
`ok` when it matches what init would write, `kept` when it differs — and never
rewritten; there is no force flag. The manifest is written last, so a project root
only appears over a whole tree, and a run interrupted at any point completes on
re-run by creating exactly what is still missing. Two inits back to back leave
every byte in the tree as the first left it.

## System Context

Init runs where no project exists yet, so it cannot lean on `check` or the archive
to protect anything — the create-only contract is the whole protection. It is also
the crash story: idempotence and interrupt recovery are one mechanism, which keeps
the verb free of state files and lock ceremony
(`a-half-init-strands-the-tree`).

## Satisfy

`Scaffold` (per artifact: absent — emit; present — read, compare, report; emission
ordered with `archi.toml` last).

- test — two inits back to back: the second creates nothing and every file's bytes are identical
- test — a hand-edited starter module and CLAUDE.md survive a re-run untouched, reported `kept`
- test — a tree holding everything but the manifest completes: the re-run creates `archi.toml` alone
