---
kind: functional
origin: stressor(the-rot-is-visible-only-at-the-crash)
satisfied-by: [Members, Cli]
deferred:
---

# check surveys the members

In a spec that declares members, `archi check` surveys the map and
reports its rot as advisory findings — never errors. Four decay modes,
each naming the row's source file (manifest path or the machine-local
overlay) and the repair verbatim:

- the mapped path does not resolve — `archi repo map <m> <dir>`
- the mapped checkout is a linked worktree — the main checkout to map
- the checkout's origin is not the declared `url` — a wrong clone
- the latest baseline sits on no branch — a squashed landing;
  `archi version anchor --repo <m>`

## System Context

The mint gates fire only when a mint happens; `repo ls` answers only
when asked. `check` is the verb every editing round already runs, so
the map's decay belongs on its worklist. A memberless project prints
nothing new; findings never block.

## Satisfy

`Members` widens its survey with the four probes; `Cli.check` folds the
survey findings into the report after the docs pass. Url comparison
normalizes protocol, user and a `.git` tail before it calls a mismatch.

- test — multi_repo_e2e: each decay mode planted in a fixture surfaces
  its finding with the source file and the repair; a healthy map prints
  nothing; exit stays 0 throughout
