---
links: [the-sweep-folds-the-symbol-the-declaration-named, a-drifted-declaration-refuses-the-wave-that-moved-it]
prefer: [evolvability]
over: [correctness]
---

# A fold may move a body, never an interface

The cleanup wave is allowed to move code a declaration named, and the repin follows. It is
not allowed to change what that code offers. The line is one the tool already computes: a
link pins the interface and the body separately, and the sweep's own probe reports `body
moved; the watched interface holds`. A fold that moves only the body repins without a
question. A fold that moves the interface refuses, and a person decides.

The alternative was to freeze anchored code. That is the rule this tree has been following,
and the last sweep showed what it costs: the same expression stood inline six times, folding
it compiled and passed, and it was left in place because each copy sat inside a body a link
anchors. The rule that keeps links still had become the rule that keeps duplication, and
declarations were about to multiply the anchors it protects. Held to the end, the cleanup
wave becomes ceremonial.

The price is that a repin over a body is routine, and a routine repin is a rubber stamp.
That is a real loss of correctness: the pair is re-attested by a machine that checked only
that the shape held. The interface test is what keeps the loss survivable — the thing a
caller can observe cannot move without a person — but it does not eliminate it, and the
first sign that the line was drawn wrong will be a repinned pair that no longer answers what
it says.

We accept that, because the opposite failure is silent and permanent: code nobody may tidy
rots in place, and the tool that was built to keep design and code together becomes the
reason they cannot be brought back together.
