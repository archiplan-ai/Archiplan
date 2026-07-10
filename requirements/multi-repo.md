# Multi-repo (spec and code in separate repositories)

Archi's default shape is one repository holding spec and code together. Real systems spread code
across repositories, and the spec then lives in its own — where capture has no code to diff, the
audit's delta source anchors a code-less tree, and an anchor cannot name a file outside the root.
Multi-repo makes the project's repositories first-class **members** while leaving the single-repo
shape the unmarked case: the model layer never learns about repositories — only code-links, the
scans and version *provenance metadata* gain a member dimension.

## Members

The manifest declares each repository the code lives in; the project's own repository is the
implicit **home** member and is never declared:

```toml
[[repo]]
name = "backend"                          # stable identity: refs and journal events carry it
url  = "git@github.com:acme/backend.git"  # provenance for humans and CI; archi never fetches
path = "../backend"                       # committed layout convention, relative to project root
```

**The name is the identity** — a rename is a journal migration, so names are short and stable;
the url keys nothing (urls drift across forks and protocols). **The path is a convention, not a
machine fact**: where a checkout actually sits is per-machine, so a gitignored
`archi/repos.local.toml` (written by `archi repo map <name> <dir>`) overrides it, local over
manifest. `archi repo ls` is the doctor: name, resolved root, reachable, clean, HEAD, baseline at
the latest version. A member that resolves to no checkout is **unreachable** — a reported state,
never a guess.

## Refs

A `<CODE_REF>` grows an optional member qualifier: `[<member>//]<file>[#<symbol>]`. Unqualified
stays the home member, so every existing ref, journal event and habit keeps its meaning; `//`
cannot collide with a normalized relative path and leaves `#` to the symbol. Anchors and spans
carry the member as an optional field — absent means home — and the journal stays append-only
with no migration. Reports render member-qualified paths (`backend//src/api.rs#serve`).

## Provenance and baselines

The version manifest keeps `commit` (home — the clean-tree, render-hash guarantee certifying the
render's sources) and gains per-member **baselines**:

```toml
commit  = "de3eb58"                       # home: the render's sources, guarantee unchanged
commits = { backend = "abc123" }          # member baselines: code as of this version
```

Save records a baseline for every mapped member whose tree is clean, and reports the omitted
ones. `archi version anchor --repo <member>` records a missing baseline post hoc under the
clean-tree guarantee — which is the strength code provenance always had: the render-hash match
never certified the code half of a commit, multi-repo merely names it. Baselines are provenance,
never a dependency ([versioning.md](versioning.md#why-this-shape)).

## Scans, capture, audit

The wave-open index walks every mapped member and keys items by qualified path; **the scan set is
recorded at wave open**, and capture diffs exactly that set — a member mapped after open is
skipped with a loud note, never silently mis-attributed. The audit runs per member against each
member's own baseline: findings are tagged with qualified paths, a member without a baseline gets
its own recovery note (commit, then `anchor --repo`), an unreachable member an unreachable note —
audit degrades per member, never globally. Absence is not drift: no scan emits observations for an
unreachable member, so evidence confidence never decays and `--prune` never retires links it
merely cannot see; verify grades them **Unreachable**, distinct from Missing. `verify --repo
<member>` scopes to one member and *does* fail on Unreachable inside that explicit scope — the
rule that makes per-repo CI gates work (each code repo's CI checks out itself plus a shallow spec
clone; the spec repo's nightly is the aggregate view).

`[audit] exclude` stays one boundary: bare patterns (`*.md`) apply in every member, a qualified
pattern (`backend//vendor/`) scopes to one. Built-ins (`archi/`, `.arch`, the manifest) apply to
home; the walker skips any nested `archi.toml` subtree inside a member — that is someone else's
project.

Git output is repo-root-relative while archi paths are project-root-relative; each member gets a
git context that resolves the repo's actual top level and rebases paths, which also fixes the
nested project root (the blessed monorepo shape) silently mismatching every audit path today.

## Why this shape

- **Not git submodules.** The pin would *be* a git dependency (violating provenance-never-
  dependency), every code commit would demand a bump commit in spec history, and the layout would
  be forced.
- **Not committed machine paths.** Identity is committed; mapping is layered: convention default,
  local override.
- **Not one archi project per code repo.** Cross-repo edges are the architecture's point; the
  graph lives whole. Spec-to-spec federation is a different, future feature.
- **Not archi fetching repos.** Credentials and network belong to CI; archi reads working trees.
- **Single-repo is the unmarked case.** Zero declarations = today's behavior byte for byte; the
  qualifier is optional everywhere it appears.

## Cross-references

- [code-link.md](code-link.md) — anchors, capture, audit; the surfaces the member dimension
  extends.
- [versioning.md](versioning.md) — provenance never a dependency; the save/anchor guarantees.
- [tasks.md](tasks.md) — the wave whose open records the scan set.
- [cli.md](cli.md) — project location; `repo` verbs, `--repo` scoping.
