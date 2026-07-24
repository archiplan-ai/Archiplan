<!-- archi:begin -->
## Archiplan

This repository is modeled with archiplan: the spec is text under `archi/`,
the model is `.arch` source under `archi/src/`, and lifecycle state moves only
through `archi` verbs — never hand-edit `archi/versions/`, the link journal,
or `closed:` stamps.

- After any model or doc edit run `archi check`: errors block, findings are
  the worklist.
- Find anything by phrase: `archi search <phrase>` — ranked hits across
  elements, intents, requirements, stressors, sessions and decisions,
  each with its address.
- Spec work delegated to subagents or workflows returns as FILES written
  under `archi/` (paths, not payloads): a finding that is not a file on
  disk does not exist, and every fan-out is gated by `archi check` plus
  a count of the files it claims to have written.
- The full workflow (model, stress, version, plan, implement with link
  capture) is the `archi` skill in `.claude/skills/archi/`; merging
  parallel spec work is `archi-merge`, and crossing a project off the
  old fractal client is `archi-migrate-fractal`.
<!-- archi:end -->
