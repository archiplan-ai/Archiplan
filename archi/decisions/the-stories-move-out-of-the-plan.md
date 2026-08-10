---
links: [two-things-are-called-a-scenario, the-wing-replaced-something-that-worked, the-plan-closes-on-the-world-s-scenarios, scenarios-close-the-plan, Planner]
prefer: [correctness]
over: [simplicity]
---

# The stories move out of the plan

The plan authors no scenarios. It collects them from the world facts covering the nodes
its tasks name, and keeps only the ceremony: print the block, latch, close.

The plan's stories were free text on the plan envelope, and free text is cheap to write
and impossible to check. They were also written per plan and died with it, so the same
behavior was restated by every plan that touched it, each time in new words and each
time unverified. Moving them to the wing costs the ease: a Gherkin block written against
a recorded condition is more work than a sentence, and a plan can no longer invent a
story at the moment it needs one.

We take that cost. It buys three things the free text never had. The block outlives the
plan, so the same behavior is stated once. The block parses, so a link from a step to a
symbol becomes possible. And the block has a reason attached to it, so a story that
stops mattering can be found and dropped through the fact that carried it, instead of
being copied forward into the next plan by habit.

The reason the stories were decoupled from the spec survives the move. A story crosses
many nodes, and `covers` is a list: a fact that conditions four nodes names four, so
nothing is pinned to one element and nothing lies about its scope.
