# The wave gate demands coverage for the node's whole surface, not the delta's

**Kind:** friction (plan gate scope) · found closing plan `close-without-minting`
**Status:** resolved 2026-07-08 — the gate presses the delta; the rest is a checklist (`gates-press-the-delta`)

`plan task add <node>` derives spec_refs as the node's entire spec surface — `Cli` yielded 11:
the node, its type edge, and one `Agent.drive` edge per verb — and the wave-close gate then
requires an *asserted* link per ref. But capture only mints candidates from the wave's delta, so
for refs the delta never touched (a one-verb fix leaves eight verb edges untouched) the gate
demands links capture cannot supply: the operator must know to hand-run
`link add <edge> <file#symbol> --kind …` nine times or the wave will not close.

## Observed

Plan `close-without-minting`, t1 on `Cli`, delta = `run_version`'s Unchanged arm: the gate listed
10 uncovered refs. The only in-delta candidates for them were false claims
(`Cli.check ← main.rs#run_version` — run_version does not implement check). Closing required
hand-adding l0258–l0266, the full dispatch table.

## Impact

First wave on any hub node pays a link-authoring tax unrelated to its change, under gate
pressure — exactly when rushed links get minted. (The tax is one-time per node — the live fold
carries the links forward — and the extracted dispatch table *is* genuinely useful traceability;
the problem is the coercion and the false-candidate temptation, not the links.)

## Fix shape

Either scope the gate to refs the wave's delta plausibly touches (capture already knows the
delta), or let `task add` scope the derived refs (`task add Cli --port version`), or keep the
gate but have it *suggest* the `link add` commands for uncovered refs — turning the jam into a
checklist. Whichever way, the gate's message should say hand-authoring is the expected move, not
leave the operator to discover `--kind` by usage error.

## Resolution

The `evidence-pressure` round (v0005, stressor `gate-demands-untouched-surface`) derived
`gates-press-the-delta`; plan `follow-the-delta` implemented it. Capture reports, per task, the
refs its claimed changed items carry signal for — the same term test that gates minting — and
the coverage gate blocks only on that pressed subset. Unpressed uncovered refs print as exact
`archi link add <ref> <file#symbol> --kind indirect` lines, on the blocked message and the
passing close alike, so the hub-node dispatch table stays available as voluntary traceability
instead of a coerced tax. A delta pressing nothing closes its wave without demanding links;
an asserted link satisfies its ref however it was born.

Observed on the fix's own wave: the gate listed 3 pressed gaps (Planner's advance/author
consults and its type edge) instead of all 11 in-flight refs; the two Planner→Links consult
edges and the node itself were already covered by standing links, and nothing was demanded for
them. The fix-shape option taken is the first one this issue proposed — scope the gate to the
delta — plus the third as garnish: the gate now *suggests* the hand-authoring it used to
extort.
