---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# One door resumes the standing work

`archi-resume` is the page for picking up work that already stands. It reads the record
before any archaeology — `archi worktree ls` with its flags for the seats, `archi status`
for the binding and the states, `archi plan list` for the lifecycles, `archi version list`
and `archi search` for anything a question names — and routes by what the reads show: a
draft plan continues in `archi-plan`, a started one in `archi-implement`, a completed one
with a standing seat lands through `archi-finish-worktree`, an open stress round or an
unsaved model goes to `archi`, and a worktree that exists only as a pushed branch
re-attaches with `archi worktree mint <slug>` before anything else. More than one standing
seat is the user's choice through the poll tool, never the agent's. A cascaded seat enters its members too: member code
is edited only in the member worktree paths `status` prints — standing ones are switched
into, absent ones re-attach with `archi worktree mint <slug> --repos ...` (it extends,
never recreates), and `archi repo ls` is the health read for the members. A member
checkout outside the session's working directories is added to them before any git runs
there. The
page chooses; each workflow skill keeps its own opening check and runs it after the choice.

## System Context

The opening protocol lives at the head of every workflow skill, and it answers "how do I
enter the seat I already know". The question it does not answer is the one a fresh session
actually has — which seats stand, in what state, and which skill continues each — so the
session improvises: reads git history first, reports on the wrong round, or stalls at a
harness boundary it misreads as a wall. The live case: a cleared session lost its seat, hit
the working-directory guard on a member checkout, called it a bug and handed its git back
to the operator.

## Satisfy

`AgentBrief` (the `archi-resume` page: the reads, the routing table, the re-attach, the
poll rule, the working-directory note). `Scaffold` (embeds and installs it byte-equal,
like the rest).

- test — a fresh init installs `archi-resume` byte-equal to the embedded copy
- test — the embedded page names the reads: `worktree ls` with `--status`, `status`,
  `plan list`, and the record-before-archaeology rule
- test — the routing table names all five: draft to `archi-plan`, started to
  `archi-implement`, completed to `archi-finish-worktree`, an open round or unsaved model
  to `archi`, and the pushed-branch re-attach through `worktree mint`
- test — the page says a member checkout outside the session's working directories is
  added to them before git runs there, and more than one standing seat goes to the user as
  options
- test — the page enters the members: switch into a standing member worktree, re-attach an
  absent one through `worktree mint` with `--repos`, read their health with `repo ls`, and
  edit member code only in the printed paths
