---
covers: [Links, Links.Grader, Links.Capture]
sources: [archi/requirements/code-link/code-link.md]
uses: []
---

# An assistant guesses which files answer a written obligation

A person hands a piece of work to an assistant and expects it to change the right files.
The assistant holds no record of which files answer which written obligation, so it reads
the tree and guesses. It guesses well often enough to be useful and wrong often enough to
cost: the person then points at the file by hand, one correction at a time. Months later,
when somebody has to recover the thread, they dig through the commit history or keep a
table of correspondences by hand, and both of those fight ambiguity forever. The behavior
follows from the guessing: what an assistant cannot know it must be told, and being told
once is worth nothing unless the telling is written down where the next reader finds it.

## What kills this

An assistant that names the files behind an obligation correctly almost every time from
the tree alone, with no record to read. The operator put the number at 99.9 percent. At
that accuracy the record is bookkeeping for a question nobody has to ask, and the whole
thread costs more to keep than to do without.

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

How often the guessing is wrong today is not measured. The operator names it as costly
from experience, and nobody has counted.
