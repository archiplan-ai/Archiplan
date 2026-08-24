---
links: [a-verification-repeats-its-requirement, the-briefing-requirement-outlived-the-briefing, DocsCompiler]
prefer: [simplicity]
over: [correctness]
---

# Prose is not checked against prose

Nothing in this repository compares one piece of written English against another. A plan's
verification may restate its requirement word for word, and a requirement may describe an
installed text that no longer says what it claims. Both passed on the day they were written
and both were found by a person reading, not by a verb.

Two mechanisms were considered and refused. A textual comparison — the plan bullet against
the requirement bullet — catches only verbatim repetition, and a restatement in fresh words
is the common case and the harder one; it would report the honest duplicate while missing
the paraphrase, which teaches exactly the wrong lesson about what the check means. A
semantic comparison needs a judgement, and a judgement in a checker is a second opinion
that cannot be argued with.

We take the cost, and it is real: two claims already went stale in one day of use, and both
were caught by accident — one because a question was asked about something else, one because
a sub-agent was disciplined enough to report outside its scope. Nothing makes those accidents
reliable.

What holds instead is where the two layers are written. A requirement is written at the spec
stage against the model; a verification is written at the plan stage against a chosen stack,
and the skill's own text says to name the test rather than repeat the claim. When they read
the same, the plan simply added nothing — a waste, not a defect. That is the reason this is
affordable to accept and the briefing case is not: a stale requirement is read as the
contract, so its cost lands on the next reader rather than on the author.
