# Multi-repo workflow

How to run the archi loop when code lives in repositories of its own and the spec repository
holds only the model, requirements, stress rounds, plans and the link journal. The loop's
rhythm — save, plan, implement, `plan next`, confirm, commit, anchor — is unchanged from the
single-repo shape; what changes is where commands find code (**members**) and how many commits
land at the end (one per touched repository).

A project that declares no members is today's project, byte for byte — nothing below applies
until the first `[[repo]]` row.

## Setup, once

Checkouts side by side, the spec repository as the archi project:

```
~/work/acme-spec/     # archi.toml, archi/, .arch sources — the project root
~/work/backend/       # code
~/work/web/           # code
```

Declare each code repository as a member in `archi.toml`:

```toml
[[repo]]
name = "backend"                          # the identity refs carry: backend//src/api.rs#serve
url  = "git@github.com:acme/backend.git"  # provenance for humans and CI; archi never fetches
path = "../backend"                       # committed convention, relative to the project root

[[repo]]
name = "web"
path = "../web"
```

The **name** is load-bearing: journal events and refs carry it forever, so keep it short and
stable — renaming it orphans every ref written under the old name (verify then says so, loudly,
naming the recovery: restore the declaration). The **url** keys nothing. The **path** is a
convention; where a checkout actually sits on one machine is machine business:

```bash
archi repo map backend ~/checkouts/backend    # writes archi/repos.local.toml — gitignored
```

`archi repo ls` is the doctor. Run it whenever anything seems off:

```
home     /Users/you/work/acme-spec  clean  9f21ab3  baseline 9f21ab3… (save)
backend  /Users/you/work/backend    clean  1bba0a2  baseline 1bba0a2 (save)
web      unreachable (../web) — archi repo map web <dir>  -  baseline -
```

Unreachable is a reported state, not an error — a half checkout is the normal state of a
multi-repo team. Every scan narrows to what it can see and says so; nothing decays, nothing is
pruned, nothing fails for a checkout that is merely elsewhere.

## The round

**1. Spec edit — in the spec repository.** Model and requirements change, then the closing
save:

```bash
cd ~/work/acme-spec
archi version save -m "close stress round: gateway-backpressure"
```

The save now reports one line per member:

```
baseline backend: 1bba0a2
no baseline for `web`: its tree is dirty — commit it, then `archi version anchor --repo web`
saved v0042 (patch, 3411 bytes, …) — close stress round: gateway-backpressure
```

You usually get the baselines for free: you were editing spec, so the members sit clean at
their last commit, and that commit is exactly "where code stood when this architecture was
agreed" — the audit's delta source later. A dirty or unreachable member is *named*, with its
recovery. Then, as always:

```bash
git commit -am "v0042: gateway backpressure"
archi version anchor                        # home provenance: clean tree + render-hash match
archi version anchor --repo web             # after web commits — see "late baselines" below
```

**2. Plan.** Task `outputs` name the files capture will attribute — qualified for member
files, bare for home:

```json
"outputs": ["backend//src/api/handler.rs", "backend//src/api/valve.rs"]
```

**3. Open the wave.** `archi plan start` snapshots home *and every mapped member* into the
wave index and records the scan set. A member mapped after the wave opened is outside that
set — capture will skip it with a note rather than diff it against an index that never saw it,
so map everything the wave will touch *before* `plan start`.

**4. Implement — in the code repositories.** No archi involvement, no commits required yet.
Capture is git-free: it reads working trees, so uncommitted member state is exactly what it
diffs.

**5. Close the wave.** From the spec repo (or anywhere with `--project ~/work/acme-spec`):

```bash
archi plan next
```

Capture rescans the recorded set, and the candidates come back qualified:

```
captured l0102-a5efbf  indirect evidence captured(t3) Gateway.Valve ← backend//src/api/valve.rs#Valve::admit
```

Review as ever — `link ls --evidence`, `link confirm` the load-bearing, `link rm` the
drive-bys, re-run `plan next`. The coverage gate, signal test and wave discipline are all
unchanged; the member qualifier is identity, never signal.

**6. Land it — one commit per touched repository.**

```bash
cd ~/work/backend    && git commit -am "gateway backpressure valve"   # the code
cd ~/work/acme-spec  && git commit -am "links+plan: gateway round"    # the record
```

This is the one honest cost of the split: the old single atomic commit is now a pair. The
journal is append-only JSONL, so parallel features union cleanly; convention carries the rest —
name the spec version in the code PR.

## Late baselines

A member dirty at save time gets no baseline. After it commits:

```bash
archi version anchor --repo backend
# anchored v0042: baseline backend at 4c1d9e2… (anchor-born — the span since the save is unaudited)
```

The entry records `born = "anchor"`, and the audit words that member's window honestly:

```
note: `backend`'s baseline is anchor-born — the span between the save and the anchor is unaudited
```

Anchor *before* implementing the round, not after — a baseline recorded after your delta puts
that delta outside the audit's window (capture still attributes it; only the aggregate
dark-delta view narrows). Home's `archi version anchor` keeps its stronger guarantee — clean
tree *and* the render hash-matching the version — because home's commit contains the render's
sources. A member baseline has the clean-tree half alone, which is all code provenance ever
had.

## The other direction: code moved without a round

A hotfix lands in a member outside any plan. Nothing captures it at write time — the hygiene
surfaces catch it, per member:

```bash
archi link verify          # asserted links whose symbols drifted — repin or fix
archi link audit
# unaccounted delta: backend//src/hotfix.rs:1-14 (in `retry_budget`) — no link claims it
# note: no delta source for `web`: commit it and run `archi version anchor --repo web`, or pass --since web=<rev>
```

Each member audits against its own baseline; `--since backend=<rev>` overrides one member's
delta source, bare `--since <rev>` means home. Account for findings with a qualified add:

```bash
archi link add Gateway backend//src/hotfix.rs#retry_budget --kind indirect
```

## Absence, precisely

| situation | behavior |
|---|---|
| `link verify`, member unmapped | links grade **Unreachable**, exit 0 — reported, never Missing |
| evidence links into an absent member | **no decay events** — absence is not an observation |
| `link audit --prune`, member unmapped | its links are neither graded nor pruned |
| `link verify --repo backend`, backend unmapped | **exit 1** — you asked for it, it must be there |
| journal names a member the manifest lost | Unreachable, note names the recovery: restore the `[[repo]]` row |

That last row is the rename trap: the member name is declared identity. If the remote moves,
update `url` and touch nothing else.

## Scan boundary

`[audit] exclude` stays one setting. A bare pattern applies in every member; a qualified one in
exactly its member:

```toml
[audit]
exclude = ["*.md", "backend//vendor/"]
```

Built-ins are unchanged (`archi/`, `.arch`, the manifest — home's tree), and a member subtree
holding its own `archi.toml` is skipped whole: that is someone else's project. Git output is
rebased into each repository's frame before any comparison, which also covers the project
rooted below its git root — the blessed monorepo nesting audits correctly.

## CI

**Spoke (per code repo — the default).** Each member's CI checks out itself plus a shallow
clone of the spec repository, maps itself, and gates on its own links:

```bash
git clone --depth 1 git@github.com:acme/acme-spec.git spec
archi repo map backend . --project spec
archi link verify --repo backend --project spec     # Unreachable here = exit 1, by design
```

**Hub (aggregate — nightly).** The spec repository's CI clones every member at the manifest's
`url`s into the conventional paths and runs the full sweep:

```bash
archi link verify --project spec
archi link audit  --project spec
```

Cheap per-PR gates at the spokes, the whole dark-delta and incidence picture at the hub. Archi
itself never touches the network — the `url` is for the clone script.

## Failure modes

- `repo ls` says unreachable → `archi repo map <member> <dir>`; the row is machine-local and
  never merged.
- `repo map` refuses: *not declared* → add the `[[repo]]` row first; the overlay maps identity,
  it never mints it.
- save says *no baseline for `m`* → commit that member, then `archi version anchor --repo m`.
  Do it before implementing, or the audit's window opens late (and says so).
- capture notes *mapped after this wave opened* → the member joins at the next wave open; its
  delta this wave is unattributed — keep it out of the wave's outputs or reopen.
- verify's wall of *unreachable* on a laptop with one checkout → correct and calm: exit 0, no
  decay. Scope with `--repo` to the member you actually have.
- a member renamed in the manifest → every old ref grades Unreachable with the
  *restore its `[[repo]]` row* note; put the old name back (rename migration is deferred, on
  record).
