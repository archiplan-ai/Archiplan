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

## 02 — Set up

### New project

```sh
cd my-system
git init && git commit --allow-empty -m "seed"   # the seat discipline stands on git
archi init
archi build
```

`archi init` scaffolds it all: `archi.toml` (with `protected = ["main"]` — branches
that never receive a local merge), a starter model, the six workflow skills in
`.claude/skills/`, the `CLAUDE.md` brief, and the `.gitignore` lines for machine-local
state. Create-only and safe to re-run. `archi build` must pass before anything else —
then open your agent and run `/archi`.

### Existing project

```sh
cd existing-repo
archi init
```

Same verb, inside the repo that already has code. `/archi` then works brownfield: it
captures the intent of the *change being asked* — not the whole legacy — recovers just
the slice of the model that change touches, and anchors the load-bearing existing code
with links from day one. From there the audit is the ratchet: wherever code moves with
no architectural account, that is where the model grows next.

### The worktree seat

All spec work rides a git worktree. The first mutating verb in an unbound checkout
mints a seat and prints where to go; `check` and `build` refuse to bless uncommitted
spec edits sitting outside one. One seat carries one whole unit — spec, then its plan,
then the code — and lands once, at the end (`archi worktree merge`). Parallel efforts
are parallel seats; code spread across repositories cascades member worktrees beside
their checkouts (`--repos`).

After a binary upgrade, run `archi sync-skills` in a project — it refreshes the
installed briefing to the new binary's copies. Every skill runs it as its first move,
so a stale briefing corrects itself.

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
