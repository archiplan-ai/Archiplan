# Audit counts repository prose as dark delta

**Kind:** friction (audit noise) · found while closing plan `fix-carrier-inference`
**Status:** resolved 2026-07-08 — `[audit] exclude` in archi.toml, one boundary for every scan (`dark-deltas-are-code`)

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

## Resolution

Took the manifest route, as the loop: `archi/stress/boundary-pressure/` pressed v0004 — the
eight-finding wall replay (breaking) plus three survivors fencing the fix
(`unclaimed-code-stays-dark`: exclusion can never eat real code silently, pinned by a regression
with a rogue `.rs` beside the excluded prose; `linked-prose-still-verifies`: exclusion scopes the
scans, never the claims — l0085/l0243 stayed clean through the change;
`capture-and-audit-agree`: one seam, both walks). Derived `dark-deltas-are-code`; implemented via
plan `scope-the-scans` @ v0004: `scan_exclusions` + `excluded` consulted by `code_files` and
`delta_hunks` (`crates/archi/src/links/mod.rs`), the manifest tolerating and strictly validating
`[audit]` (`crates/modeling-lang/src/source/project.rs` — a typo inside the section is
`E_PROJECT`), patterns: `dir/` prefix, `*.ext` glob, exact path. This repository excludes
`*.md`.

The payoff was diagnostic: with the wall gone, the audit surfaced eleven honest *code*
dark-deltas from earlier rounds that the prose had been burying — the exact skim-past failure
the issue predicted, demonstrated on the tool's own history. Capture minted zero prose
candidates for this round's wave (105 candidates vs 164 last round on a comparable surface).
Second consecutive behavior-only round closed by `version save` itself.
