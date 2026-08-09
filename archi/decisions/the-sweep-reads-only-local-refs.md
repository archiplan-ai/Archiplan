---
links: [the-receiving-branch-is-stale-after-the-merge, Seats.Landing]
prefer: [simplicity]
over: [operability]
---

# The sweep reads only local refs

Archi never reaches the network on its own, and the sweep keeps that
rule: it proves integration against the local receiving branch and
nothing else. The cost is real — a pull request merged on the forge
stays invisible until someone pulls, so a seat can stand for days after
its work landed.

We take that cost. A tool that fetches on a listing command needs
credentials, a timeout policy and an offline story on every verb. The
listing says what it compared against, so the operator reads "not here
yet" instead of a verdict about the forge, and the skill — which may
use `gh` — is where a question to the forge belongs.
