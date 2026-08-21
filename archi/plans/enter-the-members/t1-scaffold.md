---
node: Scaffold
owns: [one-door-resumes-the-standing-work]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the member-seat block on the resume page

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-resume.md
- crates/archi/tests/init_e2e.rs

## Stack

- one block joins "The seat entry" of `skills/archi-resume.md`, before the
  working-directories sentence: a cascaded seat enters its members too — member code is
  edited only in the member worktree paths `status` prints, never in a member's main
  checkout; a standing member worktree is switched into like the home one; an absent one
  re-attaches with `archi worktree mint <slug> --repos a,b` — it extends the seat, never
  recreates it, and a refused baseline routes through `--base <member>=<branch>` as
  `archi.md` teaches; `archi repo ls` is the health read on the way in — reachability,
  cleanliness, baselines
- the reads list gains `archi repo ls` as its member line (a short clause or its own
  numbered entry — match the list's voice); verify every CLI line against
  `./target/release/archi` usage before writing it
- the existing four resume guards keep their assertions; one new guard per the new verify
  bullet, in the file's `flat()` + contains style
- `scaffold.rs` untouched: the page is embedded by path

## Verifications

### one-door-resumes-the-standing-work

- test — the page enters the members: switch into a standing member worktree, re-attach an
  absent one through `worktree mint` with `--repos`, read their health with `repo ls`, and
  edit member code only in the printed paths
