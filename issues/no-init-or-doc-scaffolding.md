# No `archi init`, no doc scaffolding

**Kind:** missing feature (onboarding) · found by the self-hosting bootstrap
**Status:** resolved 2026-07-09 — `archi init` shipped (intent `cold-start`, round `cold-start-pressure` @ v0011, plan `stand-up-a-project`, commit cd94884); the `archi new` half is a recorded deferral (`skeletons-come-from-a-verb`)

Bootstrapping a project is entirely hand-rolled: `archi.toml`, the `src/` layout, and every doc
under `archi/` were authored from the specs (and, in practice, from reading
`docs/schema.rs` — the specs alone leave schema corners open). Nothing scaffolds:

- `archi init` — manifest + `archi/src/` + a passing empty model;
- `archi new intent|requirement|epic|session|stressor <name>` — a file with the right slug-derived
  name, all frontmatter fields present-but-empty, reserved sections in the right order.

## Impact

The doc schema is strict on purpose — every field present (empty is a state, absence is not), a
YAML subset with no quoting and inline-only lists, exact `System Context`/`Satisfy` and
`Attractor`/`Resolution` ordering, H1 must slugify to the filename. Each rule is individually
good; together they are a gauntlet every new file re-runs. All of it is mechanical, which is the
definition of scaffolding work. The error messages are excellent, but the loop is
write → check → fix when it could be generate → fill.

## Fix shape

An `init` verb and a `new` verb that emit schema-perfect skeletons (slug computed by the same
`md::slugify`, fields empty, sections in order, a placeholder summary line). Both are pure
file-emission — no new semantics — and would also give agents a canonical example of each format
in-repo instead of in the spec.

## Resolution

Shipped the project half of the fix shape, wider than sketched: `archi init [<dir>]` stands a
directory up whole — the source dir with a commented starter module compiling to zero statements,
the workflow and merge skills installed verbatim under `.claude/skills/`, a fenced archi block in
`CLAUDE.md`, and `archi.toml` written last so a project root only ever appears over a whole tree.
Create-only per artifact and idempotent: what exists is read and reported (`ok` byte-equal,
`kept` divergent), never rewritten; an interrupted run completes on re-run; an existing manifest
routes the starter through the compiler's own reader (`modeling_lang::source::manifest_src` —
one parser, no drift against `build`), and a parse failure aborts an untouched tree.
`crates/archi/src/scaffold.rs` + `run_init` in `main.rs`; 8 unit + 7 e2e tests. The doc half —
`archi new` skeletons — is deliberately deferred with its reason on record
(`archi/requirements/cold-start/skeletons-come-from-a-verb`): one create-only verb proves the
emission discipline first.
