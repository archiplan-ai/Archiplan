---
version: v0003
closed: v0004
---

# Order pressure

Third round, second issue replay (`issues/carrier-inference-order-dependence.md`): the resolver's
single pass makes name binding a function of walk order. The pressures here press the bug from
both sides — the replay itself, a variant that rules out the shallow fix (sorting modules
differently), and a survivor that the real fix must not regress.
