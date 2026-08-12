---
kind: functional
origin: intent
satisfied-by: [PlanFile, Planner]
deferred:
---

# A record bullet may wrap

A bullet in a plan record may run onto further lines. A non-empty line that opens no bullet,
no heading and no fence, standing under a bullet inside a bullet section, continues that
bullet: the lines join with one space and parse as one bullet. This is the lazy continuation
CommonMark already defines, so a record reads in any markdown viewer as it parses here. A
blank line ends the bullet. Prose sections are untouched, because a bare line there is prose
and always was. An error still names the line the bullet opened on.

## System Context

The parser reads line by line and asks each line for a `- ` prefix. Nothing recorded that
rule and nothing chose it: it is what a line-at-a-time reader does when nobody writes the
continuation case. A wrapped bullet is valid markdown, so the record refused a file that
every other reader accepts, and the refusal pointed at the continuation line while restating
the grammar of the bullet — which sent the reader to re-read a bullet that was already
correct.

The workaround was prose in the planning skill telling the author not to wrap. A rule that
exists to keep a parser comfortable, carried in a skill, is the shape this repository
already calls out elsewhere: the tool asks the writer to work around it. That line goes with
the fix.

The join is one space and not a newline, because a bullet is one value — a path, a ref, a
`from t<N> — note` — and its readers print it on one line. Keeping the break would make the
same record parse differently depending on where the author's editor wrapped.

## Satisfy

`PlanFile` (the charter and the task record fold continuations before they read a bullet, in
bullet sections only). `Planner` (`plan verify` reports the bullet's own opening line).

- test — a stack bullet wrapped onto a second line parses as one bullet, joined by one space
- test — the same for spec, inputs, outputs, architecture and verification bullets
- test — a blank line ends the bullet, and the next `- ` opens the next one
- test — a heading or a fence under a bullet ends it and is read as itself
- test — prose sections keep every line, wrapped or not, exactly as today
- test — a malformed bullet is still refused at the line it opened on, not at a continuation
- test — a file with no wrapped bullet parses byte-identically to today
