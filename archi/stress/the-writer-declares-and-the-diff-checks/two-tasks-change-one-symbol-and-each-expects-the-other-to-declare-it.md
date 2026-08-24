---
affects: [Links.Capture, Planner]
outcome: accepted
---

# two tasks change one symbol and each expects the other to declare it

Run a wave where two tasks claim the same file and both edit the same symbol.

The gate reads "every changed symbol is named". Named by whom? The diff is one set for the
whole wave; the declarations are one file per task. A symbol two tasks both touched can be
named by either, by both, or by neither, and each rule breaks differently. Neither: both
sub-agents assumed the other owned it, and the wave refuses on a symbol both actually
worked on. Either: the first file to name it satisfies the gate, and the second task's
contribution leaves no record. Both: two asserted pairs on one symbol, and no reader can
tell whether that is two genuine answers or one duplicated.

The wave already knows about this case — capture marks such files `shared` — but marking is
not deciding, and today nothing depends on the decision because candidates are cheap.

## Attractor

The rule that gets written is "either", because it is the one that never blocks a wave. The
record then attributes a shared symbol to whichever sub-agent happened to finish first,
which is a coin toss, and the reverse view reports that coin toss as the answer to who
built what.

## Resolution

The pressure was answered once with a rule — every claimant declares the shared symbol — and
the answer was withdrawn before it shipped, together with the per-symbol accounting it
depended on. The gate asks only that a task in flight has written a file that declares
something; nothing walks the delta asking who owes what, so there is no owing set for two
tasks to divide.

The consequence stands unmitigated and is accepted. A symbol two tasks changed may be
declared by one, by both or by neither, and the record says whatever the writers wrote. What
buys that is the cost the wider rule turned out to carry: applied to the wave that built it,
it demanded thirty entries for two files, twenty-seven of them for tests and the constants
inside them. A gate that expensive is not paid, it is worked around.

The first sign this was wrong will be shared work whose record names one author.
