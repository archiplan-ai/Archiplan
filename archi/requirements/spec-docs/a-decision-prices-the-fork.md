---
kind: functional
origin: intent
satisfied-by: [DocsCompiler, Tradeoffs]
deferred:
---

# A decision prices the fork

One decision file carries one trade: a name, the rationale prose, and the machine
fields of its price — `links`, what the trade is about, speaking both reference
currencies at once (doc slugs and live model elements, every entry checked,
`E_DOC_REF` otherwise); `prefer`, the axes it buys; `over`, the axes it pays. Axes
come from a fixed, code-defined nine (`archi axes`) plus any off-list label kept
verbatim — legal, surfaced as the `off_list_axis` finding, never rejected: a
recurring off-list share is the signal the set no longer fits the project, and
editing the nine is a release-level migration, never a project edit. Both axis
lists are zero-or-more (empty is a valid non-comparative record); the same axis on
both sides is `E_DOC` — a trade has two sides. A decision is atomic: no sections,
no anchor, one flat file per trade under `archi/decisions/`, its slug joining the
project-wide currency.

## System Context

The stress loop is otherwise purely additive — every breaking stressor derives
requirements, so the design only accretes "good". Decisions make deliberate badness
visible and priced instead of implicit: the agent frames both branches of a
breaking fork in axis labels, the user picks the direction, and the pick is a
revealed priority. `archi tradeoffs show` tallies that revealed profile beside the
declared stance (`priorities-weight-the-read`) — descriptive only, the declared
configuration alone weights the read. Retrieval carries decisions as cards and
inverts their links onto element cards (`archi search --kind decision`).

## Satisfy

`DocsCompiler` (the decision schema, link resolution in both currencies, the
two-sided-axis contradiction) and `Tradeoffs` (the axis vocabulary and the
revealed tally).

- test — docs::a_decision_prices_the_fork_and_deviations_are_loud
- test — docs::decision_slugs_join_the_currency_and_off_list_axes_surface
- test — tradeoffs::decisions_reveal_the_lived_priorities
- test — search::decisions_are_cards_carrying_their_trade
