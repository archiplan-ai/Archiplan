---
node: Scaffold
owns: [one-door-resumes-the-standing-work]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the resume page and its guards

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-resume.md
- skills/archi.md
- crates/archi/src/scaffold.rs
- crates/archi/tests/init_e2e.rs

## Stack

- `skills/archi-resume.md` is new: the freshness header naming
  `.claude/skills/archi-resume/SKILL.md`, the `archi-search` pointer line, then three
  parts — the reads, the routing table, the seat entry
- the reads, in order: `archi worktree ls [--status active|closed|all] [--plan <slug>]
  [--spec <effort>]` (the seats; a `waiting` row is work in flight and is never closed),
  `archi status` (the binding, the plan state, the version state, the open round, the
  member worktree paths), `archi plan list` (every plan with its lifecycle), and for
  anything a question names, `archi version list` and the `archi-search` page — the
  record before archaeology: git history is the last resort, and reading it first is what
  produces a report about the wrong round
- the routing table, state to skill: plan `draft` — finish authoring in `archi-plan`;
  plan `started` — `archi-implement` picks the wave up; plan `completed` with the seat
  standing — land through `archi-finish-worktree`, or continue a new unit in the same
  binding; an open stress round or an unsaved model — the `archi` skill, at that stage; a
  worktree existing only as a pushed branch — re-attach with `archi worktree mint <slug>`
  (it attaches, never creates), then route by its plan state; nothing standing at all —
  new work through `archi`
- the seat entry: `cd` into the chosen worktree yourself (the CLI never changes your
  directory); more than one standing seat is one poll question with the seats as options,
  never the agent's own pick; a member checkout outside the session's working directories
  is added to them first (`/add-dir` in Claude Code, the equivalent elsewhere) — git
  refused there is the session's boundary, not the tool's, and handing git back to the
  operator is not the repair; the chosen workflow skill then runs its own opening check
- `skills/archi.md`: the "Opening: find your worktree" section gains one pointer sentence
  — picking which standing unit to resume, and which skill continues it, is the
  `archi-resume` page; the section's own discipline stays word for word
- `scaffold.rs`: `SKILLS` eleven -> twelve; `init_e2e.rs`: install count 15 -> 16 if that
  is what the current assertion holds — read it, do not assume; `EMBEDDED_SKILLS` gains
  the row, the pointer and confirm-candidates guards pick the page up automatically; new
  guards per the verify bullets

## Verifications

### one-door-resumes-the-standing-work

- test — a fresh init installs `archi-resume` byte-equal to the embedded copy
- test — the embedded page names the reads: `worktree ls` with `--status`, `status`,
  `plan list`, and the record-before-archaeology rule
- test — the routing table names all five: draft to `archi-plan`, started to
  `archi-implement`, completed to `archi-finish-worktree`, an open round or unsaved model
  to `archi`, and the pushed-branch re-attach through `worktree mint`
- test — the page says a member checkout outside the session's working directories is
  added to them before git runs there, and more than one standing seat goes to the user
  as options
