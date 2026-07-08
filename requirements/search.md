# Search

Given a natural-language phrase, return the ranked list of KB objects most related to it —
model elements (nodes, rel/conn types, views — with their identity prose), intents,
requirements, stressors and sessions — each hit a card with a snippet and its kind's
next-hop references. `archi search <phrase>` on the [CLI](./cli.md).

## Results output

Ranked cards. Every card carries the object's address — the slug or model path, ready for
piping into `archi query` / `archi incidence` / the editor, plus `file:line` for doc
objects — a score, the best-matching snippet, and kind-true references: an element card
its definition, the requirements satisfied by it, the stressors affecting it and its edge
neighbors; a requirement card its origin, satisfied-by and open/satisfied/deferred state;
a stressor card its session, affects and outcome; a session card its pin and closing seal.

## Matching

Deterministic lexical scoring, no persisted index: both sides normalize (lowercase,
identifier splitting on case, dots and hyphens), a shared prefix of three or more
characters covering at least half of the longer token matches at half weight (so
`versioning` reaches `versions` without a stemmer), term weights are damped by document
frequency computed over the corpus in the same pass (a term in every card contributes
nothing), and a hit in a name outweighs the same hit in a summary or body. The order is
total: score, then kind, then slug — the same tree and phrase always answer byte-identically.

## Working-copy included

Search keeps no derivative of the corpus: every query scans the live tree and the freshly
compiled model, so a mutation is searchable in the immediately following call — no
`version save` in the loop, and search can never disagree with `archi check` about what a
file is.

## Degradation

Each corpus fails alone. A model that does not compile darkens the element cards only —
doc cards still answer, and the report's `dark` field names the missing corpus with its
first diagnostic; a doc file that fails its parse degrades to a raw-text card. The verb
exits 0 with whatever it could search: search is the orientation verb for a broken tree.

## Interaction with versions (Search * Versioning)

Deferred. The archive seals the canonical render — the model alone; requirements, stress
rounds and sessions have no archived form, so a `--at <id>` over the full corpus cannot be
honored: pinned elements beside live prose would answer with a chimera wearing a version
label. Search takes no `--at` until doc sources version alongside the render
(`agent-retrieval/versions-stay-searchable`, deferred with this reason).
