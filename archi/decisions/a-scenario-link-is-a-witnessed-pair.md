---
links: [a-scenario-link-binds-two-hashes, the-scenario-is-the-address-not-the-step, Links.Canonizer]
prefer: [correctness]
over: [operability]
---

# A scenario link is a witnessed pair

A link from a scenario to code records a digest of each side. Either side moving fails the
link until somebody binds the pair again. This reverses half of an earlier decision, which
held that rewording a step leaves the link untouched.

That earlier call was made against a risk that is real: step text is prose, people rewrite it
for taste, and a link that decays on taste teaches an operator to ignore decay. We keep the
half of it that still holds — a step is not addressable, and the address is the scenario name.

What the earlier call missed is the order in which the two sides are written. The scenario is
written first and the code is generated to answer it, so the pair is true at birth. Afterwards
either side moves alone, and the dangerous case is the quiet one: the code changes under an
unchanged scenario, every reference resolves, and the promise and the answer have parted with
nothing to say so. A pointer cannot notice that. A witnessed pair can.

We pay in false alarms. A step reworded for clarity will fail a link over code nobody touched,
and the operator will look, see that the behavior did not move, and repin. That is the cost of
the pair meaning anything at all, and the alternative — a link that survives every edit to both
sides — is a link that survives being wrong.
