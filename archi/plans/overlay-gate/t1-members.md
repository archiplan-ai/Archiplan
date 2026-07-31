---
node: Members
owns: [a-mapping-names-the-main-checkout]
---

# t1 — Members

`archi repo map <member> <path>`: before the row is written, resolve
the path and ask git whether it is a linked worktree — `rev-parse
--git-dir` differs from `--git-common-dir` there. If it is, refuse:
name the branch standing in that worktree, say the mapping outlives
it, and print the repo's main checkout (first row of `git worktree
list --porcelain` of that repo) as the path to map instead. The main
checkout itself, and any plain non-worktree checkout, maps as before.
The overlay file format does not change.

## Spec

- `Members`
- `Function type_of Members`
- `Archive.locate recall(<-MemberSet) Members.resolve`
- `Cli.drive consult(->Command, <-Report) Members.survey`
- `Links.locate recall(<-MemberSet) Members.resolve`
- `Links.locate_scan recall(<-MemberSet) Members.resolve`
- `Seats.locate recall(<-MemberSet) Members.resolve`

## Inputs

## Outputs

- crates/archi/src/members.rs
- crates/archi/tests/multi_repo_e2e.rs

## Stack

- git rev-parse --git-dir / --git-common-dir via the existing Command plumbing
- git worktree list --porcelain — the main checkout the refusal names

## Verifications

### a-mapping-names-the-main-checkout

- test — multi_repo_e2e: `repo map` onto a linked worktree refuses; the message carries the standing branch and the main checkout path; mapping the main checkout succeeds unchanged
