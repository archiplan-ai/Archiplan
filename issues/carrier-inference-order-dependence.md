# Carrier inference is module-name-order dependent

**Kind:** bug (compiler) · found by the self-hosting bootstrap

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
