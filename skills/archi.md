---
name: archi
description: Drive the archiplan spec workflow — capture intent, derive requirements, model, harden by stress, version. Use when you architect a system with archiplan, greenfield or brownfield. Planning is the archi-plan skill. Execution is the archi-implement skill.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

# Archi workflow

## Ground rules

**Everything is text.** The model is `.arch` source under `archi/src/`.
Requirements, stressors and decisions are markdown under `archi/`. You
edit the prose in the files, but the commands make the skeletons. `archi req
add|rm` and `archi stress open|add|rm` create and retire the records.
Each command writes every machine field and leaves the text slots for you.
The plan is a folder of records like the rest of the spec. Commands do
creation, removal and lifecycle: `plan use`, `plan task add|rm`, `start`,
`next` and `close`. You edit prose and curation in the files, and `plan
verify` is the list of work to do. Lifecycle moves only through commands.
Run `archi check` after every editing round. Errors block. Findings are
the work to do.

**Search, do not grep.** `archi search <phrase>` is ranked retrieval over
every archi object: model elements with their identity prose, intents,
requirements, stressors and sessions. Each hit carries its addresses
(file:line, satisfied-by, affects, state), so the next command starts there.
Grep misses the model, because definitions live in the compiled graph and
not on disk as prose. Narrow the search with `--kind`. Machine-read it
with `--json`. Search before you derive a requirement, to see whether a
claim like it exists. Search before you define an element, to see whether
the concept is already modeled. Search when a finding names something
unfamiliar.

**Show, do not tell.** When the user asks you to explain or to visualize
the design, pipe a query into the visualizer: `archi query <filters> |
archi viz`. It draws the subgraph as a readable ASCII diagram. It
collapses detail and deep nesting. It refuses a slice too large to read
and gives hints to narrow it.

**Never invent references.** Requirements name model elements by absolute
path. Stressors pin versions. Tasks pin nodes. `check` and `plan verify`
verify every reference. A broken reference is a bug you just created.

**Harden first, execute second.** You write code against a *pinned*
version, never against a moving spec.

**Files are the only return channel for spec work.** A requirement, a
stressor or a model element exists when its file exists. It never exists
as data in a chat, as a JSON return, or in the memory of an orchestrator.

## Two modes: solo and orchestrated

**Solo** is the default. One actor runs the loop below as written: edit
the files, run `archi check` after every round.

**Orchestrated** applies when the harness pushes workflows or subagents,
as ultracode does. The artifacts and the loop stay the same. One thing
inverts: the return channel.

1. Every delegate's prompt names the exact files it must WRITE. One
   stressor is one file, and that is what makes parallel work safe:
   parallel files never conflict. The delegate returns the list of paths
   it wrote plus one summary line, and nothing more. Findings that come
   back without a file on disk do not exist.
2. Delegates write content files only: stressor files, requirement files
   and separate `.arch` modules. Lifecycle commands belong to the
   orchestrator alone. These are `version save`, session open and close,
   and every `plan` and `link` command. Some files take one writer only, and
   that writer is the orchestrator. These are the session charter, a
   plan's record folder, and edits to one shared module. A plan's
   `state.json` is never hand-edited.
3. **Check the files.** After every fan-out, before anything else, the
   orchestrator verifies the result on disk. Run `archi check`, then
   count the artifacts against what the fan-out claims: `archi search
   --kind stressor`, and `ls` on the round's folder. A fan-out that
   returned findings but wrote no files did not happen. Write the files
   now, or run the delegates again.

IMPORTANT: Do not do the whole cycle below in one pass. Ask the user
which stage to work on, and do only that stage. Examples: initial
architecture and stress for a greenfield project, stress and a model
update, the plan, the execution of the plan.

IMPORTANT: Guide the user through the archiplan session. Do the steps in
order. Before each step, ask the user to pick one of two modes. (1) You
complete the step autonomously and summarize it. (2) You collaborate:
propose, discuss, and execute only after you agree. After each step,
offer the next directions. Ask every question through the editor's poll
tool (AskUserQuestion in Claude Code, the equivalent elsewhere). Never
write a freeform question when the answer is a choice.

IMPORTANT: Keep free text in the spec short.

## Opening: find your worktree

One worktree carries one whole unit of work: the spec, then its plan,
then the code. The unit merges once, at the end. Do these steps at the
start of every working session, before any mutation.

1. Run `archi status`. It prints the checkout, its branch, its binding,
   the version state, the open stress round, and every plan with open
   lifecycle. Beside it, before the session's first `archi check`, run
   `archi check-update`. It prints one line. When it names a newer
   version, tell the user and continue. Never install unasked.
   **"not a git repository" is a full stop.** Isolation, branches and one
   clean landing all need git. Put ONE question to the user through the
   poll tool (AskUserQuestion), with exactly two options and no default. **Create the
   repository**: `git init` plus a first commit, then start again at step
   1. Or **cancel the whole session**. There is no third path. Never
   continue bare. Never mutate an ungoverned tree.
2. **A worktree that already binds this checkout continues.** `status`
   can show this checkout bound. When it does, and the user asks for more
   work, put one question through the poll tool with two options.
   **Continue the unit here.** A finished plan does not end the worktree.
   A new round, a save and `archi plan use <name>` join the same binding,
   and the landing later carries it all at once. Or **land the worktree
   now**: run `archi worktree merge <slug>` (the archi-finish-worktree
   skill) and make a fresh worktree for the new work.
3. **Look for the existing worktree first.** Run `archi worktree ls`.
   Narrow it with `--spec <effort>` or `--plan <slug>`. Work on a spec
   continues in that spec's worktree, so `cd` there. A plan made current
   there (`archi plan use <name>`) joins the same binding and pins the
   spec version of that branch. A worktree that exists only as a pushed
   branch re-attaches with `archi worktree mint <slug>`. That attaches
   the worktree. It does not create one.
4. **Create a worktree only for work that no worktree carries.** Run
   `archi worktree mint <slug> [--plan <name>]`. The CLI creates the
   branch `archi/<slug>`, the sibling worktree and the registry entry,
   then prints the path. `cd` into it yourself, because the CLI never
   changes your directory. A mutating command in an unbound checkout refuses
   with the same choices: the worktrees that stand, or the command that
   creates one.

Multi-repo work cascades. Derive the participating members from the spec
and the plan through task outputs, spec refs and links. Then extend the
worktree with `archi worktree mint <slug> --repos a,b`. A second mint
extends the worktree. It never recreates it. Edit member code only in the
member worktree paths that `archi status` prints, never in a main
checkout. Member branches start from the recorded baseline commit of the
pinned version. When that baseline is not on the checkout's branch,
`worktree mint` refuses and lists candidate branches. Relay the choice
through the poll tool and run the command again with `--base
<member>=<branch>`. A first pass with no recorded baseline refuses the
same way. `archi version anchor --repo <member>` records a baseline from
the member's clean checkout, and `--base` names the branch directly. The
tool never guesses the main branch of a member.

Immediately after a mint with `--repos`, check the new member worktrees.
Run `git log --oneline <base>..HEAD` in every member worktree the command
printed. A new worktree shows nothing. Relay anything it lists to the
user verbatim before any work starts. `archi check` findings report the
decay of the member map: stale rows, wrong clones and stranded baselines.
Read them. They are the work to do.

The registry moves only by the commands `archi worktree ls` and `archi
worktree drop`. Never move it by hand. To close a worktree, run `archi
worktree merge <slug>` (the archi-finish-worktree skill). Merge a spec
early, before the rest of its unit, in one case only: another effort that
depends on yours must pin your published version. The default unit stays
in one worktree and lands once.

## Greenfield

1. **Init.** `archi init` scaffolds it all: `archi.toml`, the source
   directory with a starter module, this skill and the CLAUDE.md brief.
   The manifest sets `protected = ["main"]`. A protected branch never
   receives a local merge, only `--to` plus a push and a PR. The
   one-worktree rule itself is unconditional and needs no declaration.
   Init is create-only and safe to run again: it reports existing files
   and never rewrites them. `archi build` must pass before anything else.
2. **Capture intent.** Use one folder per problem area:
   `archi/requirements/<intent>/<intent>.md`. Write a name and the
   problem statement in the user's own terms. Do not solution here. One
   question does belong here: what is this project willing to be bad at?
   Record the answers as decisions under `archi/decisions/` with `prefer`
   and `over`. They are the first entries of the recorded priorities.
3. **Derive requirements.** One claim is one file, and the command makes it:

   ```
   archi req add "<title>" --intent <folder> --kind functional|non-functional --origin intent
   ```

   Every parameter is explicit. A missing parameter is a refusal. An
   unknown intent lists the folders. `--deferred <reason>` is the only
   optional flag, and its absence *is* the state. The command writes the
   exact schema shape — frontmatter, `System Context`, `Satisfy` — and
   leaves the text slots empty. `check` holds a requirement without its
   summary until you write the prose. Write the summary first. Write the
   context and `Satisfy` as the elements land. `archi req rm <slug>`
   retires one requirement, and it refuses while a plan owns the slug.
   Any other heading in the file opens a subrequirement. Leave
   requirements open: `unsatisfied_requirement` findings are work to do,
   not errors.
4. **Draft the model.** Read the ontology first with `archi query --top`.
   The unclassified nodes are the types of the preset, and each one
   carries its definition. Classify every term against them (`Service
   type_of AuthService`) or against types you define. Then write nodes,
   ports and typed edges in `.arch`. The syntax is in "`.arch` in brief"
   below. As elements land, fill each requirement's `satisfied-by`, its
   Satisfy prose, and its verification bullets (`- test — …`, `-
   type-level — …`). Run `archi check` until it reports zero errors. A
   passing check closes with the NKP scoring line and the refactoring
   directions. `archi nkp` prints the full landscape report. Read the
   scoring line like this:
   - N is the count of components in the landscape. E is the count of
     couplings between them.
   - K̄ is the mean count of couplings per component. It says how many
     components one change touches on average.
   - σ is the spread of that coupling. A high σ against K̄ means that a
     few nodes hold most of it. Those nodes surface as hotspots, and they
     are the highest-risk refactoring targets.
   - P̄ is the mean neutrality. It is the share of the design that can
     move with no global ripple.
   - The regime is one of three. ORDERED keeps changes local. CRITICAL
     (K̄ 1–3) is the target: changes propagate without cascading. CHAOTIC
     makes every change ripple, so decompose the hotspots before you
     refactor.
5. **Save.** `archi version save -m "<why>"` seals the render.
6. **Stress.** Run an adversarial round against the version you just
   saved.

   **What the round writes.** The round writes stressors, verdicts,
   derived requirements, decisions and model edits. It never writes
   application code or tests. A verdict that implies code work becomes a
   derived requirement for the plan, not work to do now.

   **Aim before you press.** Read the landscape of the version you just
   saved: the NKP scoring line, `archi nkp --hotspots`, and the
   `under_stressed` findings of the previous round. Hotspots and
   unpressed terms take the first stressors.

   Open the session with the command: `archi stress open "<title>"`. It pins
   the version you just saved. A moved model refuses and points to
   `version save`. The command derives the folder from the slug, and it
   refuses while another round is open. Then write the charter paragraph
   in the new file: what this round presses, and why now.

   *For each stressor — identify, attractor, verdict:*

   a. **Identify.** Look past the happy path, think lateraliry. Pick a stakeholder, a
      failure mode, a scale concern, or a regulatory or operational
      constraint that the happy path ignores. The valuable stressors
      cross a boundary that the architecture treats as separate. Create
      the stressor in the open round with `archi stress add "<title>"
      --affects <A,B,...>`. The affects resolve against the round's
      pinned version at the write, and one message names every miss.
      Then write the description in the new file. The first line is
      imperative. After it, add the structure a reader needs. One
      stressor is one pressure and one file. A whole round of skeletons
      lands in one `archi batch -` call. A wrong stressor retires with
      `archi stress rm <slug>`, in an open round only. Derived
      requirements hold a stressor in place.
   b. **Attractor.** Name the configuration the system gets pushed
      toward. Write it as a markdown body.
   c. **Verdict.** There are three outcomes: `surviving`, `breaking` and
      `accepted`. Record it in `outcome:`, which stays `pending` until
      the round decides. `Resolution` is non-empty exactly when the
      verdict is in. A survivor is not irrelevant, because the matrix
      still records the pressure.


   ```markdown
   ---
   affects: [AuthService, AuthService.Storage]
   outcome: breaking
   ---

   # <Stressor title>

   <stressor_description>

   ## Attractor

   <attractor_description>

   ## Resolution

   <description_of_solution>: derived `<requirement_1>` and `<requirement_2>`.
   ```

   `affects` is mandatory and non-empty. It holds absolute paths that
   name terms or types of the *pinned* version, and a type covers every
   term it classifies. `check` resolves them against that version and not
   against the live tree, so later edits never orphan a round. `outcome`
   stays `pending` until the round decides, then becomes `surviving`,
   `breaking` or `accepted`. `Resolution` is non-empty exactly when the
   outcome is decided. It states why the design held, what the answer is,
   or what consequence you keep. Affects stand either way: they record
   where you applied the pressure, not how it went.

   Only a breaking stressor gives the user a choice, because a survivor
   teaches nothing about priorities. When a stressor breaks, do not
   derive a requirement silently. Present both options, price both of
   them in axis labels, and let the user pick the direction:

   `Fix <solution> costs <axes>. Accept <consequence> sacrifices <axes>
   and preserves <axes>.`

   You articulate the axes on each side. A fix is not free virtue: it
   pays in new operational surface, in a higher K̄, and in spent budget.
   The user's pick is a recorded priority. Never derive it
   automatically. The axes are a fixed nine, and `archi axes` lists them
   with their definitions. Any other label is legal and kept verbatim,
   and `check` surfaces it as `off_list_axis`. Recurring off-list labels
   mean the set no longer fits the project. The choice prices real
   trade-offs. It is not busywork. A cheap fix that advances an axis the
   project already keeps needs no choice. Make it and report it.

   *Fix* sets `outcome: breaking`, and the break demands answers. Derive
   requirements, one per concrete obligation, with `origin:
   stressor(<slug>)`. Mid-session requirements answer pressure. They
   never open new intents. Model edits go into the live tree. A breaking
   stressor that no requirement records as its origin is the
   `breaking_unanswered` finding. A fix that paid something real earns a
   decision that records its price.

   *Accept* sets `outcome: accepted`, and nothing derives. An origin that
   names an accepted stressor is an error. `Resolution` states what you
   live with. The sacrifice lives on a linked decision. To accept without
   one is the `accepted_unjustified` finding. A break is never accepted
   silently. One decision may sign several accepted stressors.

   **Scope in a multi-repo model.** The ask of the user can name one
   repository. When a stressor's answer reaches into other members, never
   widen the scope silently. Put one question through the poll tool with
   two options: **extend the work to those repos**, or **stay with the
   repositories the user named**. Record declined scope as a derived
   requirement with `--deferred <reason>`. Never make a quiet model edit
   or a quiet worktree extension for a repo the user did not name.

   A decision is one file under `archi/decisions/`. The folder is flat
   and the filename is the slugged name. The decision is the only file
   that carries axes:

   ```markdown
   ---
   links: [<stressor-slug>, <element-or-slug>, …]
   prefer: [simplicity, cost]
   over: [reliability]
   ---

   # <Decision title>

   <the rationale — why this trade, in the user's terms>
   ```

   `links` names what the trade is about in both kinds of reference: doc
   slugs and live model elements. Every entry is checked. `prefer` and
   `over` take zero or more axes, and an empty pair is a valid
   non-comparative record. The same axis on both sides is an error.
   Everything is correctable after the fact: edit the file, then run
   `check` again. `archi search --kind decision` retrieves the decisions.
   `archi tradeoffs show` tallies the revealed profile — what the
   recorded trades actually chose — beside the declared stance.

   The next `version save` mints the version that carries the answers,
   closes the session, and prints the incidence report. It does this
   whether the model changed or not. A behavior-only round closes against
   the version it pressed, mints nothing, and exits 0. An accepted break
   is still a break there: the row stands in the matrix and presses on.
   `archi incidence` replays the findings. Read them, severest first:
   - `compound_vulnerability` (alert) — two stressors that still stand,
     surviving or accepted, together cover everything that satisfies an
     intent requirement. The promise breaks only in combination. A pair
     with an accepted member is flagged louder, because part of the joint
     break was signed off.
   - `density_alert` (alert) — the matrix is denser than τ_K. Stress
     lands everywhere at once.
   - `boundary_crossing_stressor` (warn) — one stressor presses far more
     terms than typical. It crosses a boundary worth making explicit.
   - `hyperliminal_coupling` (warn) — two terms react together with no
     declared path between them. This is a hidden dependency. Add the
     edge, or split the shared concern.
   - `stress_hotspot` (warn) — one term takes a τ_D share of the round.
     It is a decomposition candidate.
   - `merge_candidate` (info) — the same co-reaction over a declared
     path. The two nodes may be one node, or they may share a concern you
     can extract.
   - `under_stressed` (info) — no stressor touches it. Aim the next round
     there.

   **Iterate the round.** Propose stressors from distinct angles: scale,
   security, regulation, failure modes, multi-tenancy, operational load,
   long-term evolution. Continue until the user agrees, through the poll
   tool, that no new ones surface. Triage the incidence findings before
   you open the next round. Each kind above prescribes its move.

   Principles of the round. Look past the happy path first. One stressor
   is one pressure. `affects` names terms or types of the pinned version,
   never edges. Surviving is not the same as irrelevant. An accepted
   break is a real verdict that must carry its decision, and incidence
   still counts it as a break. `version save` closes the round, and until
   then the report is incomplete.

   Repeat steps 4 to 6 until a round survives. That version is the
   hardened spec. After that final `version save`, put one question to
   the user through the poll tool with two options: **commit the spec
   work now** on the worktree's branch, or **leave the tree as it is**.
   Never commit unasked.
7. **Plan.** Use the `archi-plan` skill. It authors the charter with a
   user-polled stack and its infrastructure, the tasks per node, the
   requirement ownership, the named verifications and the scenarios.
   `plan use` refuses on an unsaved model, so save first. To execute the
   plan, use the `archi-implement` skill.
8. **Steady state.** Run `archi check` and `archi link verify` in CI. Run
   `archi link audit` for code that moved with no spec account, spec that
   no code realizes, and decayed evidence.

## Brownfield

The system exists. You *recover* the model, you do not invent it, and you
author code-links from day one.

1. Init as above, inside the existing repo.
2. Capture the intent of the **change being asked**, not of the whole
   legacy. The intent scopes what you model.
3. Recover the model. Read the code. Model only what the intent touches,
   plus its boundaries, and keep the neighbors as single nodes. Write
   requirements for the observed behavior that must not break, beside the
   new asks.
4. Anchor reality. Run `version save`, commit, then `archi version
   anchor`. A bootstrap saves on a dirty tree, so provenance — the delta
   source of the audit — needs the anchor afterward. Then run `archi link
   add <element> <file#symbol> --kind indirect` for the load-bearing
   existing code. Asserted links make `link verify` and `link audit`
   meaningful immediately. Use `indirect` by default. Use `literal` only
   where the exact body is the contract.
5. Stress the recovered model as in greenfield. Legacy assumptions are
   the best stressors.
6. Plan with the `archi-plan` skill, and execute it with the
   `archi-implement` skill. A task over an existing node seeds its
   incoming edges, which are the contracts not to break. Declare every
   file you will touch in `outputs`, so capture attributes your delta
   instead of reporting leftovers.
7. The audit is what keeps the model honest. An `unaccounted_delta`
   finding means code moved with no architectural account. Grow the model
   where the findings cluster.

## `.arch` in brief

One file is one module. Its module path is the dotted relative path under
`archi/src/`, so `archi/src/flows/login.arch` is `flows.login`. The
offside rule applies: a line that ends in `:` opens an indented block.
Use spaces only, because tabs reject. Write one statement per line.
Comments start with `//`. A comment that trails a `def` or a `port` line
attaches as that element's **definition**. A standalone comment block
that abuts a `def` from above attaches the same way. A definition is one
sentence of identity prose of 240 characters or fewer. Obligation
vocabulary rejects — `must`, `should`, `shall`, `ensures`, `handles` —
because obligations belong in requirement docs. Comments on `open`, edge
and application lines stay free. This is the whole surface:

```
def view login_flow

def rel has_pii := (Service type_of *) -> (Data type_of *)
def conn login := * ->LoginForm, <-Token *   // request/response: the forward lane carries LoginForm, the reverse lane Token
def conn store := * ->(Data type_of *) *     // one-way; the payload is any Data-classified node

// The service guarding the credential boundary.
def node AuthService:
  port handle_login // receives the submitted credential pair
  def node Storage:
    port save

def node UI:        // the human-facing client
  port login
def node LoginForm  // the credential pair as submitted
def node Token      // the session grant returned on success

Service type_of AuthService    // rel edge: ends are whole node paths
Data type_of LoginForm
Data type_of Token
AuthService has_pii LoginForm  // a user-defined rel edge, shaped by has_pii's patterns

open AuthService:              // re-opens a scope, even from another file; no port decls here
  def node Handler:            // the login work itself
    port handle
    port keep
  Handler.keep store(LoginForm) Storage.save   // conn edge: an end's last segment is the port
  handle_login = Handler.handle                // application: outer port realized by a direct child's port

UI.login login AuthService.handle_login in login_flow   // carriers inferred: login's lanes are exact nodes
```

The rules the compiler holds you to:

- **Modules** — a cross-module reference needs `import mod` or `import
  mod (A, B)`. Imports gate visibility only. They are order-free, and
  cycles are legal. One name has one definition site project-wide, and
  rel and conn share a namespace. To restate edges and applications is
  free.
- **Ports** — declare a port only in the node's `def` block. `open` adds
  children, edges and applications. It never adds ports. Every port that
  an edge or an application names must be declared. A declared port that
  nothing wires is the `unused_port` finding, not an error.
- **Conn lanes** — the direction is initiation, and the payload slots
  ride the lanes: `* -> *`, `* ->P *`, `* ->P, <-Q *` (request and
  response), `* ->, <-Q *` (pull), `* <-> *`, `* <->P *`. A leading `<-`
  and a lone `<-` are illegal. A reverse lane on `<->` is illegal.
- **Conn edges** — each end is `Node.port`, and the last segment is the
  port. Carriers go in parentheses after the type name. Omit a lane whose
  pattern is an exact node, because it is inferred. Name the lane when
  the pattern is `*` or classified. Write the carrier bare when exactly
  one lane carries. Two carriers tag their lanes: `->X, <-Y`.
- **Rel edges** — the ends are whole node paths and carry no ports. `def
  rel trans r := …` marks r as transitive. A slot pattern is `*`, a path,
  or a classified `(Type type_of *)`. Any edge joins views with `in v1,
  v2`.
- **Applications** — an application is `outer = Child.port`. The right
  side is the port of a direct child. The left side is bare inside the
  node's own block, and `Node.port` elsewhere. The outer port refuses to
  delegate until a connection attaches to it (`E_NO_OUTER_PORT`).
- **Resolution** — a path's first segment resolves in this order: the
  innermost block's children, then the enclosing blocks outward, then the
  file scope. The children are semantic, wherever the project defines
  them. The file scope holds the own defs, the imports and the preset.
  Everything lowers to absolute paths.
- **Reserved** — `import def open node view rel conn port trans in` are
  not names. The preset names, `type_of` and the ontology types, are
  ambient. Never import them and never redefine them.

## Multi-repo

Code spreads across repositories while the spec lives in its own. Declare
each code repo as a **member** in `archi.toml`:

```toml
[[repo]]
name = "backend"          # the identity refs carry: backend//src/api.rs#serve
url  = "…"                # provenance for humans and CI; archi never fetches
path = "../backend"       # committed convention; archi repo map overrides per machine
```

`archi repo ls` reports the health of each member: the resolved root,
reachability, cleanliness, head and baseline. `archi repo map <member>
<dir>` writes the gitignored machine-local overlay. An unqualified ref
stays in the project's own repo, so a memberless project behaves as
today's does, byte for byte. An absent checkout is *Unreachable*: archi
reports it and never decays it. Only `verify --repo <member>` turns
absence into a failure. A save records a baseline for every clean mapped
member. `version anchor --repo <member>` records a missed one after the
fact, and marks it as anchor-born.

## Failure modes

- `link audit` notes no delta source. The last save happened on a dirty
  tree, as every bootstrap does. Commit, then run `archi version anchor`
  to record the commit as the provenance of the latest version.
- Audit findings name prose files: issues, READMEs, docs. This is not
  code motion. Mute the boundary once with `[audit] exclude = ["*.md",
  …]` in `archi.toml`. Capture and the audit share the setting, and links
  into excluded files still verify.
- `plan use` refuses. The model has unsaved changes, so run `version
  save` first.
- `worktree merge` refuses a stale member baseline, because the worktree
  tip is past the recorded mark. Run `archi version anchor --repo
  <member>` in the worktree, then run the merge again.
- `check` after a merge says the manifest holds conflict markers. Two
  branches minted the same version id. Keep the first-landed entry and
  its patch file. Then run `archi version remint -m <note> --session
  <slug>` to re-mint the later round onto the merged lineage and to
  re-stamp its `closed:`. Review the semantic delta of the merge first
  with `archi version diff <latest> live`.
- Journal merges concatenate through the union attribute instead of
  conflicting. `link verify` surfaces the notes it absorbed, marked
  `journal:`. Read them. Each one is one writer's op landing on a record
  the other writer had already retired.
- `check` after a merge says a session is *claimed by two charters*. A
  same-slug merge fused two rounds, and the markers are the signal. Never
  add `merge=union` to `archi/stress/`, because then the fusion would
  commit itself silently. `archi session fold <slug> -m <note> [--keep
  theirs]` normalizes it, and both charters stay under `## Folded:`. When
  two sessions are *both open*, run `archi session fold <loser> --into
  <winner> -m <note>`. A fold refuses across pins and across mixed open
  and sealed pairs. Split those by hand.
- A *folded round awaits remint* finding. A fused **sealed** pair was
  folded. Run `archi version remint -m <note> --session <slug>` to
  re-stamp the folded stamp, because the surviving stamp is already true.
  Remint and save refuse while markers remain. The order is archive,
  fold, remint.
- `plan next` is blocked on coverage. This is not an error. It is the
  loop: confirm or retire the candidates it just created, then run it
  again.
- Verify notes "no longer resolves at Working". The spec advanced. Run
  `plan repin`, then fix the tasks it flags.
- Never hand-edit lifecycle state (`state`, `closed_waves`, latches), the
  version archive, or the link journal. Use the commands only.

To merge parallel spec work, use the `archi-merge` skill.
