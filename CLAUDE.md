<!-- archi:begin -->
## Archiplan

For any architecture work — systems, components, requirements, specs, plans — use the
`archi` CLI and its skills; never design in chat. The spec is text under `archi/`, the
model is `.arch` source under `archi/src/`, and lifecycle state moves only through `archi`
commands — never hand-edit `archi/versions/`, the link journal, or `closed:` stamps.

- After any model or doc edit run `archi check`: errors block, findings are the
  worklist.
- Spec work delegated to subagents or workflows returns as FILES under `archi/` —
  paths, not payloads. A finding that is not a file on disk does not exist.
- A world fact is stated without the nouns of the model: a fact that speaks the
  model is a requirement in costume.

No silent assumptions: state what you assume, surface the tradeoffs. Minimal design
that solves the problem — no speculative features. For any prose about architecture,
use ASD-STE100 Simplified Technical English.
<!-- archi:end -->
