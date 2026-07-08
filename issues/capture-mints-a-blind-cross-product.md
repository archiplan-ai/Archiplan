# Capture mints a blind symbol × spec_ref cross-product

**Kind:** friction (link review ergonomics) · found closing plan `close-without-minting`
**Status:** resolved 2026-07-08 — candidates carry signal: term overlap gates the mint (`candidates-carry-signal`)

`link capture` pairs every symbol in the wave's delta with every spec_ref of the owning task, no
content signal consulted. On a hub-node task the product explodes: t1 on `Cli` carried 11 derived
refs, the delta held ~15 symbols (one changed function, a new test file's helpers and test fns,
three prose docs), and capture minted **164 candidates** — l0094–l0257 — of which 158 were
retired unread-by-any-standard-but-mine. Test-file plumbing (`MODEL`, `NEXT`, `ok`, `run`,
`temp_project`) × 11 refs is noise by construction: no reviewer will ever confirm
`Agent.drive … Cli.nkp ← version_e2e.rs#temp_project`.

## Impact

The confirm-or-retire review is the ratchet's quality gate, and a 164-candidate wall trains the
operator to mass-retire — the same skim-past disease `audit-counts-repo-prose-as-dark-delta`
documents for audit findings. The six load-bearing pairs (fix site, four regressions, the spec
doc) were indistinguishable from the wall except by prior knowledge.

## Fix shape

Put any signal at all between symbol and ref before minting: term overlap between the ref's
surface (node, ports, edge endpoints) and the symbol's name/hunk; or mint per changed *hunk*
rather than per symbol for helpers nothing references; or cap candidates per symbol to refs whose
terms appear in the containing file. Failing inference, let the plan help: a task's
`stack_mapping` already names which tech serves which node — capture could weight refs by it.
`--json` can keep the full product for tooling either way.

## Resolution

The `evidence-pressure` round (v0005, stressor `hub-wave-floods-review`) derived
`candidates-carry-signal`; plan `follow-the-delta` implemented it. Capture now mints a
(changed item, spec_ref) pair only when the ref's surface terms — split on case and
underscores, `type_of`'s own tokens excluded — overlap the item's symbol path or canonical
body tokens (file-level items add their path terms). No-signal pairs are suppressed, not
subtracted: counted in the render, whole under the new `link capture --task <T> --json`, and
free for a hand `link add` any time. Touches, decays, retirement and leftovers are unchanged.

Observed on the fix's own wave: 37 minted, 92 suppressed where the old product would have
minted all 129 — and the suppressed set was exactly the plumbing class this issue names
(`tests` module items × unrelated verb edges). The review that followed confirmed 12 and
retired 25, all readable in one screen.
