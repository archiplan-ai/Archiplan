# No `archi init`, no doc scaffolding

**Kind:** missing feature (onboarding) · found by the self-hosting bootstrap

Bootstrapping a project is entirely hand-rolled: `archi.toml`, the `src/` layout, and every doc
under `archi/` were authored from the specs (and, in practice, from reading
`docs/schema.rs` — the specs alone leave schema corners open). Nothing scaffolds:

- `archi init` — manifest + `src/` + a passing empty model;
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
