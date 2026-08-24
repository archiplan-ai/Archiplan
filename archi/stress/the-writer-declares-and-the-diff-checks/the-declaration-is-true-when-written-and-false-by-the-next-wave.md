---
affects: [Links.Grader, Links.Journal, Planner]
outcome: breaking
---

# the declaration is true when written and false by the next wave

Declare a true pair, then change the code under it two waves later.

The declaration file is input, not a home: it is written once, consumed by `plan next`, and
never read again. What survives is the link. So the moment a later wave rewrites the symbol
for a different reason, the link stands on a pair that no longer holds, and the only signal
is drift — advisory today, 184 rows of it in this tree, none failing.

The proposal makes this worse before it makes it better. Under inference the pair was a
guess and nobody leaned on it. Under declaration the pair is the record of what the code is
for, and the reverse view is rendered from it. A stale record that is read is more dangerous
than a stale record that is ignored.

## Attractor

Drift accumulates behind an interface that reports coverage. The `check` line stays green
because drift never blocked, the reverse view keeps naming code that moved on, and the
repin nobody was asked for is the one thing that would have caught it. The system converges
on the same failure it was built to prevent — a written claim that the code has quietly left
behind — with the difference that it now looks authoritative.

## Resolution

Drift stops being advisory for declared rows: derived
`a-drifted-declaration-refuses-the-wave-that-moved-it`. The refusal lands on the wave that
moved the symbol, because the person who just rewrote it knows whether it still answers what
it answered, and nobody reading the audit three months later does.

Confining the refusal to declared rows is what makes it landable at all. The 184 rows
standing advisory in this tree were made by inference and nobody leaned on them; promoting
all of them at once would wall the next wave anybody runs. What the round does not settle is
the repin itself — `a-fold-may-move-a-body-never-an-interface` grants it over a body, and
that grant is the loophole this pressure would use if the interface test is ever drawn
loosely.
