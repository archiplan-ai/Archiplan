---
kind: functional
origin: intent
satisfied-by: [Compiler.Lexer, Compiler.Definitions]
deferred:
---

# Definitions parse from comments

The comments already written in spec files are the definition surface: the trailing comment on
a defining line — `def node`, `def view`, `def rel`, `def conn`, `port` — or the standalone
comment block sitting immediately above it attaches to that element as its definition. A blank
line detaches: separated blocks, section commentary and edge annotations stay invisible prose.
One definition site per element — a trailing comment and an adjacent block on the same element
is a located error, `open` blocks take none, and a whitespace-only comment is no definition at
all.

## System Context

The sources already speak this way — nearly every node and port in this repository carries a
trailing identity comment — but the lexer drops comments at tokenization, so the prose is
invisible to everything downstream: dumps, queries, renders, archived versions. The feature
rides the existing syntax; no new marker, and every `.arch` file that compiles today still
compiles.

## Satisfy

`Compiler.Lexer` (captures every comment with its line and position alongside the token stream
instead of discarding it; the token stream itself is unchanged). `Compiler.Definitions` (its
attach pass pairs comments with elements by position — same line for a trailing comment, an
abutting standalone run for a block — walks nested defs and ports alike, and rejects the
ambiguous both-forms case instead of guessing).

- test — lexer: trailing and standalone comments are captured with their lines; the token stream is unchanged
- test — attach: definitions land on nodes, ports, views, rels and conns from both positions; a blank line detaches the block above
- test — attach: a trailing comment plus an adjacent block on one element is one located error; `open` lines and whitespace-only comments attach nothing
- test — compile: this repository's own sources compile with every existing trailing comment attached as its element's definition
