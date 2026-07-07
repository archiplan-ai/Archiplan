# Carrier inference is module-name-order dependent

**Kind:** bug (compiler) · found by the self-hosting bootstrap
**Status:** resolved 2026-07-07 — two-phase resolver (`uses-see-every-def`)

Conn-edge carrier inference fails when the edge's module sorts before the module defining the
conn type. `resolve.rs` collects definitions and resolves uses in one pass over sorted modules
(`collect_uses`, `crates/modeling-lang/src/source/resolve.rs:632`), and `edge_carriers`
(`resolve.rs:936`) looks the conn def up in `resolution.conns` mid-pass — so an edge in a module
that sorts earlier finds no def, inference for exact lanes silently degrades, and the un-carried
statement reaches the engine.

## Observed

`def conn invoke := * ->Command, <-Report *` in `src/conns.arch`; the edge
`Agent.drive invoke Cli.check` in `src/agent.arch` (`agent` < `conns`). Compile fails with the
raw engine error localized at the edge:

    src/agent.arch:15:1: E_CARRIER_REQUIRED: `invoke` carries a node on its forward lane:
    every instantiation names `carrier`; expected {"node":"Command"}

Renaming the module to `operator.arch` (sorts after `conns`) makes the same source compile. The
workaround and a pointer to this issue live as a comment in `src/operator.arch`.

## Impact

Breaks the source-format promise that compilation is order-independent ("import order carries no
evaluation semantics"). The failure mode is worse than the bug: the diagnostic is the engine's
JSON-shaped message instead of the compiler's targeted hint (`name it — invoke(->X)`), so the
author is steered toward tagging carriers rather than toward the real cause.
`compilation_is_deterministic_under_source_order` permutes discovery order, which the sorted walk
normalizes away — module *names* are never permuted, so the suite cannot catch this.

## Fix shape

Split the pass: collect every module's rel/conn defs into `resolution` first, then resolve uses.
A regression test should define a conn in module `z` and instantiate it with inferred carriers in
module `a`. This is a natural first `archi plan` over `Compiler.Resolver` — the node already
carries asserted code-links.

## Resolution

Run as the full loop, spec before code. Stressed in `archi/stress/order-pressure/` against v0003:
the rename replay (breaking), a same-module def-after-use variant (breaking — the bug was never
about module *names*; it killed any fix that just re-sorts modules), and the def-less preset conn
as the survivor the fix must not regress. Derived
`archi/requirements/self-hosting/uses-see-every-def.md`; the model phased `Compiler.Resolver`
into `Defs`/`Uses` with `Uses.read_table recall(DefTable) Defs.collect`; v0004 closed the round.

Implemented via plan `fix-carrier-inference` @ v0004 (first `archi plan` exercised end to end):
`collect_type_defs` sweeps rel/conn/view defs across all modules before `collect_uses` walks
DefNode/Open/Edge/App (`crates/modeling-lang/src/source/resolve.rs`, pass 2a/2b). Four
regressions: cross-module inferred carriers, same-module def-after-use, def-less conns unchanged,
module-renaming batch invariance. Wave capture minted the links; confirmed l0033/l0036
(collect_type_defs, collect_uses ← Compiler.Resolver) and l0039/l0040 (#resolve ← the type_of and
resolve-edge refs); the four regression tests ride as evidence. A genuinely uninferable lane now
fires the compiler's `name it — invoke(->X)` hint, never the engine's JSON error.

Verifying the last scenario surfaced a successor:
`issues/canonical-render-edge-order-depends-on-module-names.md` — inference survives the
`agent.arch` rename, but the canonical render's edge order does not, so the rename (and this
file's workaround comment) waits on that fix.
