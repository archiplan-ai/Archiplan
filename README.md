# Archiplan

**Unslop your software**

Archiplan is a software hardening environment: a CLI (`archi`) and a small modeling
language that turn software architecture from a stale diagram into a living, verified
artifact. You model the system as source code, attack the design while failure still
costs nothing, and ship code that stays verifiably tied to the spec it implements.

![](archiplan.svg)

## What you get

### An architecture that compiles

The model is source code: nodes, ports, typed connections and relations, each element
carrying one sentence of identity prose. It lives in your repo, splits into modules,
reads in review like any other diff — and it *compiles*. Every reference is resolved
and checked.

```
def conn login := * ->LoginForm, <-Token *   // request/response: the form goes out, the token comes back

// The service guarding the credential boundary.
def node AuthService:
  port handle_login   // receives the submitted credential pair

def node UI:          // the human-facing client
  port login

def node LoginForm    // the credential pair as submitted
def node Token        // the session grant returned on success

UI.login login AuthService.handle_login
```

Diagrams-as-code tools render pictures; archi compiles a model. Rename `Token` and the
build breaks at everything that names it — connections, requirements, code links. The
spec cannot rot silently.

### Requirements wired into the build

Requirements are small markdown files, one claim each, and every one names the model
elements that satisfy it — an architectural commitment made before any code exists. The
compiler verifies every name, so a requirement can't quietly point at something that no
longer exists. Unsatisfied requirements stay visible as findings: a worklist you can see,
not a TODO you forgot.

### A stress room for your design

You wouldn't ship code without tests. Archiplan gives the *design* its tests: a stress
round pins a snapshot of the model and attacks it — traffic bursts, scale cliffs,
regulatory shifts, hostile stakeholders. Each pressure is written down, aimed at named
parts of the design, and resolved *surviving* or *breaking*. A breaking pressure demands
an answer — new requirements, model changes — and stays flagged until it has one; each
round closes with an incidence report of where pressure landed and what it found. You
loop until a round survives. What comes out is a hardened spec: a design that has already
lived through its worst day, at the cost of an afternoon on paper instead of an incident
in production.

### A map of where change is safe

Archi scores the model as a fitness landscape — a technique borrowed from complexity
science. One line tells you how far the average change ripples and which regime the
design is in: **ordered** (changes stay local), **critical** (the evolvable edge — the
target), or **chaotic** (everything cascades). The full report names the coupling
hotspots — the components where risk concentrates — and the neutral corridors, whole
regions safe to restructure freely. Refactoring priorities stop being a matter of taste.

### Versions of meaning, not files

A version is a snapshot of what the design *means* — content-addressed, minted only when
the meaning actually moves, sealed against tampering, immune to git rewrites, and cheap
enough to keep forever. Diff any two versions — or a version against the live tree — and
read a semantic delta, not text churn. Stress rounds, plans, and code links all pin
versions, so "what exactly did we agree to?" always has a reconstructable answer.

### Plans that can't drift from the spec

An implementation plan pins a hardened version: code is written against a frozen spec,
never a moving one. Cut a task per element to build, and its spec references and
requirements arrive by lookup — nothing is retyped, so nothing can be mistyped. A plan
refuses to start until every requirement it touches carries a verification, then
execution proceeds in waves with a gate between each.

### Code with receipts

As each wave of code lands, archi captures what actually changed and turns it into
candidate links between spec elements and the code symbols that realize them — already
attributed to the task that produced them. You confirm the load-bearing ones, discard the
drive-bys, and the wave doesn't close until the evidence does. From then on traceability
is machine-checked: verification re-hashes every link in CI and grades the drift, and the
audit sweeps for dark code (moved with no architectural account), dark spec (claimed but
never realized), and evidence gone stale. Not a document asserting the code matches the
design — a build that fails when it doesn't.

### Meets your system where it is

No big-bang adoption. On an existing codebase you capture the intent of the change being
asked — not the whole legacy — recover just the slice of the model that change touches,
and anchor the load-bearing existing code with links from day one. From there the audit
is the ratchet: wherever code moves with no architectural account, that is where the
model grows next.

### Built for the agent era

Everything is text — the model is a small language, requirements and stressors are
markdown, the plan is JSON — so humans and agents edit the same files, and every
lifecycle move goes through a verb that validates it. Agents get ranked search over every
object in the project and a structured JSON read surface; a hallucinated reference
becomes a build error, not a buried lie. Archiplan ships with a skill that drives the
whole loop end to end, so the discipline lives in the tool — not in anyone's memory,
human or model.

### One spec, any number of repos

Code spread across repositories is the normal case, not an afterthought. Declare each
code repo as a member and links carry repo-qualified identities; every version records
where each member's code stood, and anything it can't record is named along with its
recovery — never silent. A checkout missing on one machine is a reported state, not an
error.

## The shape of the loop

```
HARDEN    intent → requirements → model → version → stress round
             ↑                                          │
             └────────── answers to what broke ─────────┘
                       …until a round survives

EXECUTE   hardened spec → plan → waves of code → confirmed evidence links
                → scenarios verified end to end → CI gates, forever
```

The through-line: each stage's output is the next stage's checked input. Requirements
name model elements the compiler verifies. Stress rounds pin versions the archive
reconstructs. Tasks pin spec elements the plan verifies. Links pin code symbols the
verifier re-hashes. Nothing is retyped — so every drift has exactly one place it can
surface, and a machine watching that place.

## Install

The repository is private, so releases install through an authenticated GitHub CLI:
each machine needs read access to `archiplan-ai/Archiplan` and a logged-in `gh`
(`gh auth login`). The steps resolve the latest release, verify the checksum, and
drop the binary on `$PATH`.

### macOS (Apple Silicon) / Linux

```sh
REPO=archiplan-ai/Archiplan
PLAT=macos-arm64                      # or: linux-x64 · linux-arm64
V=$(gh release view -R "$REPO" --json tagName -q .tagName | sed 's/^v//')

gh release download "v$V" -R "$REPO" -p "archi-$V-$PLAT.tar.gz*"
shasum -a 256 -c "archi-$V-$PLAT.tar.gz.sha256"   # Linux: sha256sum -c
tar -xzf "archi-$V-$PLAT.tar.gz"
mkdir -p "$HOME/.local/bin"
install -m 755 "archi-$V-$PLAT/archi" "$HOME/.local/bin/archi"
```

Make sure `~/.local/bin` is on your `PATH` (`export PATH="$HOME/.local/bin:$PATH"`
in your shell rc). Confirm with `archi --version`.

### Windows (PowerShell)

```powershell
$REPO = "archiplan-ai/Archiplan"
$V = (gh release view -R $REPO --json tagName -q .tagName) -replace '^v',''
gh release download "v$V" -R $REPO -p "archi-$V-windows-x64.tar.gz"
tar -xzf "archi-$V-windows-x64.tar.gz"
# copy archi-$V-windows-x64\archi.exe into a directory on %PATH%
```

> If the repository is ever made public, the scripted installer works with no auth:
> `curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.sh | sh`
> (PowerShell: `irm https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.ps1 | iex`).

## First steps

Run `archi init` in a project directory. It scaffolds everything — config, a starter
model, and the `archi` skill: a complete operating manual for the loop that your coding
agent can follow end to end.

Then go deeper:

- [skills/archi.md](skills/archi.md) — the full workflow, greenfield and brownfield, and the modeling language in brief
- [docs/versioning.md](docs/versioning.md) — what a version is and why the archive is durable
- [docs/multi-repo-workflow.md](docs/multi-repo-workflow.md) — the loop when code lives in repositories of its own
