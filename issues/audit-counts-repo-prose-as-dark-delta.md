# Audit counts repository prose as dark delta

**Kind:** friction (audit noise) · found while closing plan `fix-carrier-inference`

`link audit`'s delta scan excludes `archi/` and `.arch` sources but nothing else that isn't code:
a new file under `issues/` surfaced as `unaccounted delta:
issues/canonical-render-edge-order-depends-on-module-names.md:1-38 — no link claims it`. README,
issue files, and any repo prose are hunks the audit expects a code-link to claim, but linking
markdown to spec elements is not what code-links mean — the "architectural account" concept does
not fit prose.

## Impact

Every documentation commit adds permanent dark-delta findings (they persist until the next
version anchor moves the baseline), training the operator to skim past `unaccounted delta` — the
one finding that must stay loud to keep the ratchet honest.

## Fix shape

Exclude non-code artifacts from `delta_hunks`, either by a small default denylist (`issues/`,
`*.md` outside code trees?) — too magical — or, better, a project-level setting in `archi.toml`
(e.g. `[audit] exclude = ["issues/", "*.md"]`) with `archi/` and `.arch` staying built-in. The
capture scan and the audit should share the exclusion so candidates and coverage agree.
