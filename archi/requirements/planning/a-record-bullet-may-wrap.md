---
kind: functional
origin: intent
satisfied-by: [PlanFile, Planner]
deferred:
---

# A record bullet may wrap

A bullet in a plan record may run onto further lines, wrapped wherever the author's editor
wrapped it. A bullet section is read whole: it splits at the lines that open with `- `, and
each piece is one bullet with its line breaks and blank lines collapsed to single spaces.
Prose sections are untouched, because a bare line there is prose and always was. An error
names the line the bullet opened on.

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

Splitting the whole section beats tracking a continuation line by line, and the difference
is one case: a paragraph left under the bullets joins the last one instead of being refused.
That is accepted. A bullet section of a plan record carries bullets and nothing else, so the
case is a typo with no legitimate reading, and the alternative — a rule about which bare
lines continue and which do not — is the rule that produced this defect in the first place.

## Satisfy

`PlanFile` (a bullet section is split at its `- ` lines and each piece is collapsed to one
line, in bullet sections only). `Planner` (`plan verify` reports the bullet's own opening
line).

- test — a stack bullet wrapped onto a second line parses as one bullet, joined by one space
- test — the same for spec, inputs, outputs, architecture and verification bullets
- test — a bullet wrapped onto three lines, and one wrapped with a blank line inside it,
  both parse as one bullet
- test — blank lines between bullets change nothing
- test — prose sections keep every line, wrapped or not, exactly as today
- test — a malformed bullet is still refused at the line it opened on, not at a continuation
- test — a file with no wrapped bullet parses byte-identically to today
