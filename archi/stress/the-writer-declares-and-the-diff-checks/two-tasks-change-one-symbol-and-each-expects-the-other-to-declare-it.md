---
affects: [Links.Capture, Planner]
outcome: breaking
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

The rule is "both", written down rather than left to whichever reading the implementation
happens to take: derived `every-task-that-touched-a-symbol-declares-it`. A symbol two tasks
answered carries two pairs, each naming its task, so the record reads as two claims instead
of one duplicated.

"Either" was the attractor precisely because it never blocks a wave, and that is what makes
it the wrong answer: it buys a green wave with a coin toss about who built what. Capture
already marks such files `shared`, so the owing set costs nothing to derive — only the
decision, which now has consequences it did not have while candidates were cheap.
