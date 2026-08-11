---
covers: [Links, Links.Grader, Links.Capture]
sources: []
uses: []
---

# An assistant guesses which files answer a written obligation

A person hands a piece of work to an assistant and expects it to change the right files.
The assistant holds no record of which files answer which written obligation, so it reads
the tree and guesses. It guesses well often enough to be useful and wrong often enough to
cost. The behavior follows from the guessing: what an assistant cannot know it must be
told, and being told once is worth nothing unless the telling is written down where the
next reader finds it.

## What people do instead

The person corrects the guess by hand: they point at the file, one correction at a time,
in every session, and the correction dies with the session. To pick a thread back up
months later somebody digs through the commit history, or keeps a table of
correspondences in a document beside the code and updates it by hand; both fight
ambiguity forever, and the table is wrong the first week nobody touches it. The
correcting survives because each round is cheap — a minute now, an afternoon a year from
now, and the price is paid by whoever comes next. Only guessing that is right 99.9
percent of the time would make the record bookkeeping for a question nobody has to ask,
and the corrections that keep arriving say it is not.

## Scenarios

### Work arrives over an obligation the assistant did not write

Given an obligation that has files recorded against it
When an assistant is asked to change what that obligation covers
Then it reads the recorded files instead of reading the tree and guessing
And a file that no record names is reported rather than assumed

### The thread is recovered after its author has gone

Given an obligation whose author no longer works here
When somebody asks which files answer it
Then the record answers
And nobody walks the commit history to rebuild the answer

## Open questions

How often the guessing is wrong today is not measured. The cost is known from working
this way and nobody has counted it.
