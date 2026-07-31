---
node: Members
owns: [check-surveys-the-members]
---

# t1 — Members

Widen the member survey with four probes and fold them into `archi
check` after the docs pass, advisory only, memberless projects silent:
path does not resolve; mapped checkout is a linked worktree (the
git-dir vs git-common-dir probe already in members.rs); `git remote
get-url origin` differs from the manifest `url` after normalizing
protocol, user and a `.git` tail (skip when either side is empty);
the latest version's baseline for the member exists but sits on no
branch (`branch --contains` empty — a squashed landing). Every finding
names the row's source — the manifest `path` or the machine-local
overlay — and the repair verbatim: `archi repo map <m> <dir>`, the
main checkout to map, or `archi version anchor --repo <m>`.

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
- crates/archi/src/main.rs
- crates/archi/tests/multi_repo_e2e.rs

## Stack

- the survey plumbing already in members.rs — extend, do not duplicate
- git remote get-url origin; git branch --contains — via the module's Command style
- the check report's advisory findings block in main.rs — the docs findings' style

## Verifications

### check-surveys-the-members

- test — multi_repo_e2e: a deleted path, a linked-worktree row, a wrong-clone remote and a squash-stranded baseline each surface their finding with source and repair; a healthy map adds nothing; exit stays 0
