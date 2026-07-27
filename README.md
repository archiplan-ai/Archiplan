# Archiplan

**Turn your coding agent into a system architect**

Connect Archiplan to your agent and the code it writes gets a real
architecture behind it — stress-tested and traceable before anything ships.

![](archiplan.svg)

## 01 — Install

Install the `archi` CLI:

```sh
curl -sSf https://archiplan.ai/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://archiplan.ai/install.ps1 | iex
```

The installer resolves the latest release for your platform, verifies the checksum,
and drops the binary into `~/.local/bin` — make sure it is on your `PATH`, then
confirm with `archi --version`. Pin a version with `ARCHI_VERSION=x.y.z`.

## 02 — Getting started

Everything after the install happens inside your agent: you talk, it drives the
`archi` CLI, the skills carry the discipline. No commands to memorize.

### New project

Open your agent in the project folder and run `/archi`. Describe the system you want
in one sentence — the agent stands the project up (repository included, with your
consent), captures the intent in your own words, derives requirements, models the
architecture, stress-tests it with you and seals a hardened version. Every choice
that matters comes back as a question with priced options — nothing is assumed
silently.

Then `/archi-plan` turns the hardened spec into an implementation plan — the agent
polls you for the stack, the test frameworks and the infrastructure — and
`/archi-implement` builds it wave by wave, sub-agents in parallel, until the plan
reports done and the scenarios have run against a live stack.

### Existing project

Open your agent in the repo and run `/archi`, then name the change you want. The
agent recovers the model from the code — only the slice your change touches, the
boundaries as single nodes — writes requirements for the behavior that must not
break, and anchors the load-bearing existing code with links from day one. From
there the loop is the same: stress, version, `/archi-plan`, `/archi-implement`. The
audit keeps score: code that moves with no architectural account is where the model
grows next.

### Multi-repo

Spec in one repository, code in several. Tell the agent which repositories
participate — it declares them as members, records where each one stood at every
version, and when the work starts it seats every member beside its checkout: the
same branch across all of them, each grown from its recorded baseline. When a base
is ambiguous, the agent brings you the candidate branches instead of guessing. At
the close, member branches land by push and PR on their own forges — never by a
local merge into your checkout — and the spec lands once, whole.

### The worktree seat

All work rides git worktrees the agent manages for you: one unit — spec, then its
plan, then the code — lives in one seat and lands once, at the end. Parallel efforts
are parallel seats. Nothing mutates your main checkout, and the tools refuse to
bless work done outside the discipline — the agent is told exactly where to go
instead.

## 03 — Every skill, by what you're doing

`archi init` drops six skills into your agent, all invoked with a slash command.
Grouped by the job at hand.

### Design the architecture

**`/archi`** — the main entry point. Guides a full architecture session with the
archiplan methodology — from a one-line problem statement, through intent and
requirements, to a hardened, versioned spec. Greenfield or brownfield. Tracks coupling
live with the NKP scoring line, steering you toward the **CRITICAL** regime — the
evolvable edge where changes propagate without cascading. Stress rounds live here too:
each round discovers stressors — failure modes, scale concerns, hostile users,
regulators — names the components each one presses, applies verdicts, and turns every
break into a derived requirement or a deliberately signed trade-off. You loop until a
round survives; that version is the hardened spec.

### Plan & build

**`/archi-plan`** — turns a hardened spec into an implementation plan, authored
entirely through the `archi plan` verbs: an envelope with a user-polled stack and its
infrastructure, one task per node, requirement ownership curated from the spec's own
reverse lookup, named verifications, end-to-end scenarios. Refuses on an unsaved model.

**`/archi-implement`** — drives the build of a started plan wave by wave, every task in
its own sub-agent, until `archi plan next` reports `DONE`. The plan stays the source of
truth — task briefs come from `archi plan task show`; every wave commits inside the
seat before the join, and code-link evidence is captured as each wave closes.

### Land & merge

**`/archi-finish-worktree`** — closes a seat: lands its spec/plan/code unit, pushes
member branches for their PRs, retires the worktree and its registry binding in one
move. A protected receiving branch lands sideways (`--to` + push + PR), never by local
merge.

**`/archi-merge`** — two branches that both mutated the spec: triage the join with
`check`, resolve version-archive collisions with `remint`, read the journal's absorbed
residue, fold concurrent stress rounds. The contract is the canonical render — git
merging clean proves nothing until the composition compiles.

### Migrate

**`/archi-migrate-fractal`** — crosses a machine and its projects off the old fractal
client: swaps the binaries, then translates each `.fractal/` project into a standing,
checkable archiplan spec with a brief of what didn't map. The old tree is never
mutated — it stays on disk as the frozen reference.

## 04 — See it in action

Run `/archi` in your agent. Archiplan walks the conversation from problem statement to
a hardened spec — services, constraints, requirements, all named, all compiled:

```
def conn login := * ->LoginForm, <-Token *   // the form goes out, the token comes back

// The service guarding the credential boundary.
def node AuthService:
  port handle_login   // receives the submitted credential pair

def node UI:          // the human-facing client
  port login

UI.login login AuthService.handle_login
```

Rename `Token` and the build breaks at everything that names it — connections,
requirements, code links. The spec cannot rot silently.

## 05 — What you get

A complete system design environment inside your agent. Architecture as code. A system
knowledge-base. Decisions you can trace.

### Pre-mortem on every design

Archiplan throws traffic spikes, partial outages, hostile users, and regulators at the
spec — before you ship code that pretended those don't exist. Anything that breaks
becomes a new requirement, or a sacrifice signed by a decision.

### Never lose context

Every requirement remembers its origin — the initial problem, a specific stressor, a
stakeholder concern. Six months later, when someone asks *why did we split this from
that?*, the answer is an artifact, not a lost Slack thread.

### Fine-tune your architecture

Archiplan flags when your design is heading toward a god-service or
microservices-for-microservices' sake — before "we should refactor" turns into "we
have to rewrite."

### Code with receipts

As each wave lands, archi captures what changed and links spec elements to the code
symbols that realize them. Verification re-hashes every link in CI and grades the
drift; the audit sweeps for dark code, dark spec, and stale evidence. Not a document
asserting the code matches the design — a build that fails when it doesn't.

### One spec, any number of repos

Declare each code repository as a member: links carry repo-qualified identities, every
version records where each member's code stood, and worktree seats cascade across all
of them. A checkout missing on one machine is a reported state, never an error.

---

Archi is in beta. It's rough in places — that's the point. We're hardening it the way
archi hardens a spec, and the fastest way to make it better is to hear what breaks for
you. Try it on a real design and tell us where it bends.

Go deeper:

- [skills/archi.md](skills/archi.md) — the full workflow, greenfield and brownfield, and the modeling language in brief
- [docs/versioning.md](docs/versioning.md) — what a version is and why the archive is durable
- [docs/multi-repo-workflow.md](docs/multi-repo-workflow.md) — the loop when code lives in repositories of its own

*Archiplan — plan mode on steroids.*
