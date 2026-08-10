---
kind: functional
origin: stressor(the-bridge-has-no-verb-to-walk-it)
satisfied-by: [Cli, DocsCompiler]
deferred:
---

# One verb walks the bridge

`archi world ls` lists the standing facts, one block each: slug, path, the nodes it
covers and whether it carries sources. `--covers <node>` narrows to the facts that
condition one element, which is the question the wing exists to answer. `--json` emits
the structured envelope. The listing reads the tree and resolves nothing against a pin.

## System Context

`the-wing-is-reached-by-traversal` traded findability by phrase for simplicity, on the
ground that `covers` is exact and blind to wording where lexical scoring is neither. That
trade is only paid off if the traversal is a verb. Without it the bridge is a convention
an operator walks by opening files, and the wing gets reached the one way the decision
said it would not be — by phrase, badly. The listing belongs on the `world` verb rather
than on `query`, which composes model slices and holds no doc.

## Satisfy

`Cli` (the `ls` subcommand of `world`, its `--covers` and `--json` flags, and the human
block per fact). `DocsCompiler` (serves the loaded facts and resolves `covers` against
the live model for the filter).

- test — `world ls` on a tree of three facts prints three blocks with slug, path and covers
- test — `--covers <node>` returns only the facts naming that element
- test — `--covers` on an element no fact names returns nothing and exits zero
- test — `--covers` on a name that no model element matches refuses and says so
- test — the JSON envelope carries slug, path, covers, sources and uses per fact
- test — a tree with no wing prints nothing and exits zero
