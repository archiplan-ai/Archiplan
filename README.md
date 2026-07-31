# Archiplan

AI cares little about architecture, but writes code faster than we read it.

Connect Archiplan to your agent and the code it writes gets a real
architecture behind it — stress-tested and traceable before anything ships.

![](assets/quickstart-new-project.svg)

## 01 — Install

Install the `archi` CLI:

```sh
curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.ps1 | iex
```

The installer finds the latest [GitHub release](https://github.com/archiplan-ai/Archiplan/releases)
for your platform, verifies its checksum, and writes the binary into
`~/.local/bin`. Make sure that directory is on your `PATH`, then confirm
the install with `archi --version`. To pin a version, set
`ARCHI_VERSION=x.y.z`.

You can also install it by hand. Every release publishes one tarball per
platform — `macos-arm64`, `linux-x64`, `linux-arm64`, `windows-x64` —
with a `.sha256` file beside it. Download one, unpack it, and move
`archi` onto your `PATH`.

## 02 — Getting started

Everything after the install happens inside your agent. There are no
commands to memorize.

### New project

Open your agent in the project folder, run `/archi`, and describe the
system in one sentence. The agent does the rest:

- intent, then requirements, then the model, then stress rounds, then a
  hardened and versioned spec
- every choice that matters comes back to you as a question with priced
  options
- `/archi-plan` writes the implementation plan. It polls you for the
  stack, the tests and the infrastructure.
- `/archi-implement` runs waves of parallel sub-agents until the plan is
  done, and runs the scenarios on a live stack

### Existing project

Run `/archi` in the repo and name the change you want:

- the model is recovered from the code — only the slice your change
  touches
- requirements pin the behavior that must not break, and links anchor the
  load-bearing code
- from there the loop is the same: stress, version, `/archi-plan`,
  `/archi-implement`

![](assets/quickstart-existing-project.svg)

### Multi-repo

The spec lives in one repository and the code lives in several. Tell the
agent which repos participate:

- each member gets a worktree beside its checkout, on the same branch,
  started from its recorded baseline
- an ambiguous base comes back as candidate branches. The agent never
  guesses.
- members land by push and PR on their forges. The spec lands once, and
  whole.

### One worktree per unit of work

- one unit — the spec, then the plan, then the code — stays in one
  worktree and lands once
- parallel efforts are parallel worktrees. Your main checkout is never
  mutated.
- the tools refuse work done outside this rule, and they say where to go

## 03 — Every skill, by the job it does

The install writes the workflow skills into your agent, and a slash
command invokes each one. They are grouped by the job at hand.

### Design the architecture

**`/archi`** is the main entry point. It guides a full architecture
session with the archiplan methodology: from a one-line problem
statement, through intent and requirements, to a hardened and versioned
spec. It works greenfield and brownfield. It tracks the coupling live
with the NKP scoring line. It steers you toward the **CRITICAL** regime,
where changes propagate without cascading.

The stress rounds live here too. Each round finds stressors — failure
modes, scale concerns, hostile users, regulators. It names the components
that each stressor presses, applies the verdicts, and turns every break
into a derived requirement or into a signed trade-off. You loop until a
round survives. That version is the hardened spec.

### Plan and build

**`/archi-plan`** turns a hardened spec into an implementation plan. It
authors the plan entirely through the `archi plan` commands. The plan holds
a charter, one task per node, curated requirement ownership, named
verifications and end-to-end scenarios. The charter carries a user-polled
stack and its infrastructure. The ownership is curated from the spec's
own reverse lookup. The command refuses on an unsaved model.

**`/archi-implement`** drives the build of a started plan, wave by wave,
with every task in its own sub-agent, until `archi plan next` reports
`DONE`. The plan stays the source of truth. The task briefs come from
`archi plan task show`. Every wave commits inside the worktree before the
merge, and code-link evidence is captured as each wave closes.

### Land and merge

**`/archi-finish-worktree`** closes a worktree. It lands the spec, plan
and code unit in one move. It pushes the member branches for their PRs,
and it retires the worktree and its registry binding. A protected
receiving branch lands sideways, with `--to` plus a push and a PR, never
by a local merge.

**`/archi-merge`** joins two branches that both mutated the spec. It
triages the merge with `check`, resolves version-archive collisions with
`remint`, reads the notes the journal absorbed, and folds concurrent
stress rounds. The contract is the canonical render: a clean git merge
proves nothing until the composition compiles.

## 04 — See it in action

### Find the failures before you ship

Archiplan throws traffic spikes, partial outages, hostile users and
regulators at the spec, before you ship code that pretended none of them
exist. Anything that breaks becomes a new requirement.

![](assets/quickstart-stress-session.svg)

### Never lose the context

Every requirement remembers its origin: the initial problem, a specific
stressor, or a stakeholder concern. Six months later, when someone asks
why you split this from that, the answer is an artifact and not a lost
Slack thread.

![](assets/quickstart-decision-trace.svg)

### Fine-tune your architecture

Archiplan flags when your design is heading toward a god-service or microservices-for-microservices' sake — before "we should refactor" turns into "we have to rewrite."

![](assets/quickstart-fine-tune.svg)

## 05 — How Archiplan fits the software development life cycle

It gives you an evolutionary approach to system design, then continuous
tracking of cause and effect:

![](assets/archiplan.svg)

---

Archi is in beta. It is rough in places, and that is the point. We harden
it the way archi hardens a spec. The fastest way to make it better is to
hear what breaks for you. Try it on a real design, and tell us what
breaks.

Go deeper:

- [skills/archi.md](skills/archi.md) — the full workflow, greenfield and brownfield, and the modeling language in brief
- [docs/versioning.md](docs/versioning.md) — what a version is, and why the archive is durable
- [docs/multi-repo-workflow.md](docs/multi-repo-workflow.md) — the loop when the code lives in repositories of its own

Licensed under the [MIT License](LICENSE).

*Archiplan — plan mode on steroids.*
