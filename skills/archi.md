---
name: archi
description: Drive the archiplan workflow end to end — capture intent, derive requirements, model, stress-harden, version, plan, and implement in waves with code-link capture. Use when architecting or implementing a system with archiplan, greenfield or brownfield.
---

# Archi workflow

Ground rules, always:

- Everything is text. Model = `.arch` sources under `archi/src/`,
  requirements, stressors and decisions = markdown under `archi/`, plan = `plan.json`. Mutate by editing files;
  lifecycle moves only through verbs. Run `archi check` after every editing
  round — errors block, findings are the worklist.
- Search, don't grep. `archi search <phrase>` is ranked retrieval over
  every archi object — model elements with their identity prose, intents,
  requirements, stressors, sessions — and each hit carries its addresses
  (file:line, satisfied-by, affects, state) so the next verb starts there.
  Grep misses the model: definitions live in the compiled graph, not on
  disk as prose. Narrow with `--kind`, machine-read with `--json`. Reach
  for it before deriving a requirement (does a claim like this exist?),
  before defining an element (is this concept already modeled?), and when
  a finding names something unfamiliar.
- Show, don't tell. When the user asks to explain or visualize the design,
  pipe a query into the visualizer — `archi query <filters> | archi viz` —
  which draws the subgraph as a readable ASCII diagram, collapsing detail and
  deep nesting and refusing a slice too large to read (with hints to narrow).
- Never invent references. Requirements name model elements by absolute
  path, stressors pin versions, tasks pin nodes — `check` and `plan verify`
  verify every one; a broken reference is a bug you just created.
- Harden first, execute second. Code is written against a *pinned* version,
  never against a moving spec.

IMPORTANT: Don't rush to complete whole cycle described below in one pass. Ask user what stage to focus on instead and do only that part: initial architecture + stress (greenfield) / stress + update architecture / plan / execute plan etc.

IMPORTANT: Guide the user through an archiplan session. Follow the steps in order. Before each step ask whether the user wants you to (1) complete it autonomously and summarize, or (2) collaborate — propose, discuss, execute only after alignment. After each step offer next directions. Ask every question to the user through the editor's poll tool (AskUserQuestion in Claude Code, the equivalent elsewhere) — never dump a freeform question when the answer is a choice.

IMPORTANT: When writing free text anywhere in the spec be short and concise.

## Greenfield

1. **Init** — `archi init` scaffolds it all: `archi.toml`, the source dir
   with a starter module, this skill and the CLAUDE.md brief. Create-only
   and safe to re-run — existing files are reported, never rewritten.
   `archi build` must pass before anything else.
2. **Capture intent** — one folder per problem area:
   `archi/requirements/<intent>/<intent>.md`, a name and the problem
   statement in the user's own terms. No solutioning here. One question
   does belong here: what is this project willing to be bad at? Seed the
   answers as decisions under `archi/decisions/` with `prefer`/`over` —
   the revealed priority profile's first entries.
3. **Derive requirements** — one file per claim in the intent folder,
   filename the slugged name: frontmatter (`kind: functional` |
   `non-functional`, `origin: intent`, `satisfied-by: []`, `deferred:`),
   summary-first prose, then the reserved sections `System Context` and
   `Satisfy`. Every field and section present in every file — empty is an
   explicit state, absence an error; any other heading opens a
   subrequirement. Leave them open — `unsatisfied_requirement` findings
   are the worklist, not errors.
4. **Draft the model** — read the ontology first: `archi query --top`.
   The unclassified nodes are the preset's types, each carrying its
   definition; classify every term against them (`Service type_of
   AuthService`) or against types you define. Then nodes, ports, typed
   edges in `.arch` (syntax: *`.arch` in brief*, below). As elements
   land, fill each requirement's
   `satisfied-by`, Satisfy prose, and verification bullets (`- test — …`,
   `- type-level — …`). Loop `archi check` to zero errors — a passing
   check closes on the NKP scoring line and refactoring directions;
   `archi nkp` for the full landscape report. Reading the scoring line:
   - N components in the landscape · E couplings between them
   - K̄ mean couplings per component — on average, how many components
     one change touches
   - σ the spread of that coupling — high against K̄ means a few nodes
     hoard it; those surface as hotspots, the highest-risk refactoring
     targets
   - P̄ mean neutrality — the share of the design free to move with no
     global ripple
   - regime ORDERED — changes stay local; CRITICAL (K̄ 1–3) — the
     evolvable edge of chaos, changes propagate without cascading (the
     target); CHAOTIC — every change ripples, decompose hotspots before
     refactoring
5. **Save** — `archi version save -m "<why>"` seals the render.
6. **Stress** — an adversarial round against the version just saved. Open
   the session, `archi/stress/<session>/<session>.md`: frontmatter
   `version:` (the pinned id) and `closed:` (empty — the closing save
   stamps it), then a name and one charter paragraph, what this round
   presses and why now. At most one session is open at a time. 
   Iteratively discover stressors. 
   *For each one:*
   a. Identify. Think hyperliminarily — pick a stakeholder, failure mode, scale concern, or regulatory constraint the happy path ignores. `affects` is required and must resolve to epistatic nodes. Stressor description is a markdown body — same rule as requirements (single-quoted, imperative first line, then any structure a reader needs). One stressor = one file.
   b. Attractor. What configuration does the system get pushed toward? Attractor description is a markdown body.


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

   `affects` — mandatory, non-empty: absolute paths naming terms or types
   of the *pinned* version (a type covers every term it classifies);
   `check` resolves them against that version, not the live tree, so later
   edits never orphan a round. `outcome` — `pending` until the round
   decides, then `surviving`, `breaking` or `accepted`; `Resolution` is
   non-empty exactly when the outcome is decided — why it held, the
   answer, or the consequence being kept. Affects stand either way: they
   record where pressure was applied, not how it went.

   Only breaking stressors fork — a survivor teaches nothing about
   priorities. When a stressor breaks, present both branches, both costed
   in axis labels, and let the user pick the direction:

   `Fix <solution> costs <axes>; Accept <consequence> sacrifices <axes>,
   preserves <axes>.`

   You articulate the axes on each side — fixing is not free virtue: it
   pays in new operational surface, higher K̄, spent budget — and the
   user's pick is a revealed priority, never auto-derived. The axes are a
   fixed nine (`archi axes` lists them with definitions); any other label
   is legal, kept verbatim, and surfaced by `check` as `off_list_axis` —
   recurring off-list labels mean the set no longer fits the project. The
   fork exists to price real trade-offs, not to add ceremony: a cheap fix
   advancing an axis the project already keeps needs no fork — make it
   and report it.

   *Fix* → `outcome: breaking`, and the break demands answers: derived
   requirements (`origin: stressor(<slug>)` — mid-session requirements
   answer pressure, never new intents) and model edits in the live tree;
   a breaking stressor no requirement records as origin is the
   `breaking_unanswered` finding. A fix that paid something real earns a
   decision recording its price.

   *Accept* → `outcome: accepted`, and nothing derives — an origin naming
   an accepted stressor is an error; `Resolution` states what is being
   lived with. The sacrifice lives entirely on a linked decision:
   accepting without one is the `accepted_unjustified` finding — a break
   is never accepted silently. One decision may sign several accepted
   stressors.

   A decision is one file under `archi/decisions/` (flat, filename = the
   slugged name), the sole carrier of axes:

   ```markdown
   ---
   links: [<stressor-slug>, <element-or-slug>, …]
   prefer: [simplicity, cost]
   over: [reliability]
   ---

   # <Decision title>

   <the rationale — why this trade, in the user's terms>
   ```

   `links` name what the trade is about in both reference currencies —
   doc slugs and live model elements, every entry checked; `prefer`/`over`
   are zero-or-more (empty is a valid non-comparative record); the same
   axis on both sides is an error. Everything is correctable after the
   fact — edit the file, re-run `check`. `archi search --kind decision`
   retrieves them; `archi tradeoffs show` tallies the revealed profile
   (what the recorded trades actually chose) beside the declared stance.

   The next `version save` mints the version carrying the
   answers, closes the session, and prints the incidence report — model
   changed or not: a behavior-only round closes against the version it
   pressed, no mint, exit 0. An accepted break is still a break there:
   the row stands in the matrix and presses on. Reading its findings
   (`archi incidence` replays them), severest first:
   - `compound_vulnerability` (alert) — two still-standing stressors
     (surviving or accepted) together cover everything satisfying an
     intent requirement: a promise that breaks only in combination; a
     pair with an accepted member is flagged louder — part of the joint
     break was signed off
   - `density_alert` (alert) — the matrix denser than τ_K: stress is
     landing everywhere at once
   - `boundary_crossing_stressor` (warn) — one stressor presses far more
     terms than typical: it crosses a boundary worth making explicit
   - `hyperliminal_coupling` (warn) — two terms co-react with no declared
     path between them: a hidden dependency — add the edge or split the
     shared concern
   - `stress_hotspot` (warn) — one term soaks a τ_D share of the round:
     a decomposition candidate
   - `merge_candidate` (info) — the same co-reaction over a declared
     path: two nodes may be one, or share an extractable concern
   - `under_stressed` (info) — no stressor touches it: aim the next
     round there

   Repeat 4–6 until a round survives — that version is the hardened
   spec.
7. **Plan** — `archi plan use <name>` (refuses on an unsaved model — save
   first). `archi plan task add <node>` per node to implement; spec_refs
   and requirements are derived, never retyped. Then edit `plan.json`:
   envelope (`problem`, `technology_stack` with provenance,
   `architecture_summary`, `stack_mapping`), per task `description`,
   `inputs` (dependencies, keyed by producing task), `outputs` (files it
   will write — capture attributes deltas through these), `scenarios`, and
   one verification per matched requirement (read the matches from
   `archi plan verify --json`). Loop `plan verify` to clean.
8. **Execute waves** — `archi plan start`. Per wave: implement each
   in-flight task inside its declared outputs, then `archi plan next` — it
   captures the delta into candidate links (signal-bearing pairs only; the
   no-signal product is suppressed, whole under `link capture --json`) and
   blocks on asserted coverage of the refs the delta presses. Review
   `archi link ls --evidence`, `link confirm` the load-bearing candidates,
   `link rm` the drive-bys (subtractions stick), and re-run `plan next`.
   Unpressed uncovered refs arrive as a suggested `link add` checklist —
   hand-author the ones whose traceability you want, skip the rest. After
   the last wave it prints the scenarios: verify them end to end, then one
   more `plan next` → `DONE`.
9. **Steady state** — `archi check` and `archi link verify` in CI;
   `archi link audit` for dark deltas, dark spec, and decayed evidence.

## Brownfield

The system exists: the model is *recovered*, not invented, and code-links
are authored from day one.

1. Init as above, inside the existing repo.
2. Capture the intent of the **change being asked**, not the whole legacy —
   the intent scopes what gets modeled.
3. Recover the model: read the code; model only what the intent touches
   plus its boundaries (neighbors as single nodes). Write requirements for
   observed behavior that must not break, alongside the new asks.
4. Anchor reality: `version save`, commit, `archi version anchor` (a
   bootstrap saves on a dirty tree, so provenance — the audit's delta
   source — needs the post-hoc anchor), then
   `archi link add <element> <file#symbol> --kind indirect` for the
   load-bearing existing code — asserted links make `link verify` and
   `link audit` meaningful immediately. `indirect` by default; `literal`
   only where the exact body is the contract.
5. Stress the recovered model as in greenfield — legacy assumptions are
   the best stressors.
6. Plan and execute as greenfield 7–8. Tasks over existing nodes seed
   their incoming edges — the contracts not to break; declare every file
   you will touch in `outputs` so capture attributes your delta instead of
   reporting leftovers.
7. Audit is the ratchet: `unaccounted_delta` findings mean code moved with
   no architectural account — grow the model where they cluster.

## `.arch` in brief

One file is one module, its module path the dotted relative path under
`archi/src/` (`archi/src/flows/login.arch` → `flows.login`). Offside
rule: a line ending in `:` opens an indented block — spaces only, tabs
reject; one statement per line. `//` comments; a comment trailing a
`def` or `port` line (or a standalone block abutting a `def` from
above) attaches as that element's **definition** — one sentence of
identity prose, ≤240 chars, and obligation vocabulary (`must`,
`should`, `shall`, `ensures`, `handles`) rejects: obligations belong in
requirement docs. Comments on `open`, edge and application lines stay
free. The whole surface:

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

- **Modules** — cross-module references need `import mod` or
  `import mod (A, B)`: visibility gates only, order-free, cycles legal.
  One definition site per name project-wide (rel and conn share a
  namespace); restating edges and applications is free.
- **Ports** — declared only in the node's `def` block; `open` adds
  children, edges and applications, never ports. Every port an edge or
  application names must be declared; declared-but-unwired is the
  `unused_port` finding, not an error.
- **Conn lanes** — direction is initiation; payload slots ride the
  lanes: `* -> *`, `* ->P *`, `* ->P, <-Q *` (request/response),
  `* ->, <-Q *` (pull), `* <-> *`, `* <->P *`. No leading or lone `<-`,
  no reverse lane on `<->`.
- **Conn edges** — each end is `Node.port`, the last segment the port.
  Carriers in parens after the type name: omit a lane whose pattern is
  an exact node (inferred), name it when the pattern is `*` or
  classified, bare only when exactly one lane carries — two carriers
  tag their lanes (`->X, <-Y`).
- **Rel edges** — ends are whole node paths, no ports. `def rel trans
  r := …` marks r transitive; slot patterns are `*`, a path, or
  classified `(Type type_of *)`. Any edge joins views with `in v1, v2`.
- **Applications** — `outer = Child.port`: the right side a direct
  child's port, the left bare inside the node's own block and
  `Node.port` elsewhere; the outer port refuses to delegate until a
  connection attaches to it (`E_NO_OUTER_PORT`).
- **Resolution** — a path's first segment: the innermost block's
  children (semantic — wherever in the project they are defined), then
  enclosing blocks outward, then file scope (own defs ∪ imports ∪
  preset). Everything lowers to absolute paths.
- **Reserved** — `import def open node view rel conn port trans in`
  are not names; preset names (`type_of`, the ontology types) are
  ambient — never import, never redefine them.

## Multi-repo

Code spread across repositories, spec in its own: declare each code repo as
a **member** in `archi.toml` —

```toml
[[repo]]
name = "backend"          # the identity refs carry: backend//src/api.rs#serve
url  = "…"                # provenance for humans and CI; archi never fetches
path = "../backend"       # committed convention; archi repo map overrides per machine
```

`archi repo ls` is the doctor (resolved root, reachable, clean, head,
baseline); `archi repo map <member> <dir>` writes the gitignored
machine-local overlay. Unqualified refs stay the project's own repo — a
memberless project is today's, byte for byte. An absent checkout is
*Unreachable*, reported and never decayed; only `verify --repo <member>`
turns absence into failure. Save baselines every clean mapped member;
`version anchor --repo <member>` records a missed one post hoc, marked as
anchor-born.

## Failure modes

- `link audit` notes no delta source → the last save happened on a dirty
  tree (every bootstrap does); commit, then `archi version anchor` records
  the commit as the latest version's provenance.
- audit dark-deltas name prose (issues, READMEs, docs) → not code motion;
  mute the boundary once with `[audit] exclude = ["*.md", …]` in
  `archi.toml` — capture and the audit share it, links into excluded
  files still verify.
- `plan use` refuses → the model has unsaved changes; `version save` first.
- post-merge `check` says the manifest holds conflict markers → two branches minted the same
  version id; keep the first-landed entry and its patch file, then
  `archi version remint -m <note> --session <slug>` re-mints the later round onto the merged
  lineage and re-stamps its `closed:`. Review the merge's semantic delta first with
  `archi version diff <latest> live`.
- journal merges concatenate (union attribute) instead of conflicting; `link verify` surfaces
  any absorbed merge residue as `journal:` notes — read them, they are one writer's op landing
  on the other's tombstone.
- post-merge `check` says a session is *claimed by two charters* → a same-slug merge fused two
  rounds (markers are the signal — never add `merge=union` to `archi/stress/`, the fusion would
  commit itself silently); `archi session fold <slug> -m <note> [--keep theirs]` normalizes it,
  both charters kept under `## Folded:`. Two sessions *both open* → `archi session fold <loser>
  --into <winner> -m <note>`. A fold refuses across pins and mixed open/sealed pairs — those
  split by hand.
- a *folded round awaits remint* finding → a fused **sealed** pair was folded; `archi version
  remint -m <note> --session <slug>` re-stamps the folded stamp (the surviving one is already
  true). Remint and save refuse while markers remain — the sequence is archive, fold, remint.
- `plan next` blocked on coverage → not an error, the loop: confirm or
  retire the candidates it just minted, re-run.
- verify notes "no longer resolves at Working" → the spec advanced;
  `plan repin`, then fix the tasks it flags.
- Never hand-edit lifecycle state (`state`, `closed_waves`, latches), the
  version archive, or the link journal — verbs only.

Merging parallel spec work: the `archi-merge` skill.
