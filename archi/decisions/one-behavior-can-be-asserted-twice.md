---
links: [both-wings-assert-the-same-behavior, the-check-counts-the-wing, a-world-fact-carries-its-scenarios]
prefer: [simplicity]
over: [correctness]
---

# One behavior can be asserted twice

A requirement's verification bullet and a world fact's scenario can assert the same
behavior of the same node. Both records are legal, both resolve, and no check compares
them.

There is no cheap detector. Comparing a sentence of prose with a Gherkin step is a
judgment, not a parse, and the pair that would be compared is exactly the pair the two
wings are supposed to produce: a node held from opposite sides. A check that flagged
the overlap would fire on the healthy case far more often than on the duplicate.

We take the cost. Two guards hold it down and neither of them is machinery. The origin
rule sends a claim to one wing before it is written — pressure from inside goes to a
requirement, a condition from outside goes to a fact — and a claim that fits both is a
sign the origin was never established. The count that `check` now prints keeps the wing
small enough that a reader meets both records. When the two do drift apart, the
detector is a person reading them, which is the same detector the wing already relies
on for truth.
