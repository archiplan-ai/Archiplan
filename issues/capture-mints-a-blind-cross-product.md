# Capture mints a blind symbol × spec_ref cross-product

**Kind:** friction (link review ergonomics) · found closing plan `close-without-minting`

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
