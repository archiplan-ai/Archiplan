---
kind: functional
origin: intent
satisfied-by: [Compiler.Lexer]
deferred:
---

# The offside rule shapes the source

A `.arch` file is UTF-8 text of one statement per line — no semicolons, no line
continuations — where structure is indentation: a line ending in `:` opens a block,
its items sit strictly deeper at one common column, blank and comment-only lines
are invisible to nesting, and a dedent must return to an enclosing level. Tabs in
indentation are rejected; indent with spaces.

Identifiers are `[A-Za-z_][A-Za-z0-9_]*`; paths join them with `.`. Reserved words
are not usable as names: `import def open node view rel conn port trans in`. (A
JSON-built model using one as a name cannot render to source — the round-trip
caveat lives with `the-surface-lowers-to-one-batch`.)

Comments run from `//` to end of line and are invisible to the token stream — but
not to the tool. A comment in definition position — trailing a defining line
(`def node`, `def view`, `def rel`, `def conn`, `port`), or a standalone block
abutting one from above, its lines joined with spaces — attaches to that element as
its definition; every other comment is free prose. A blank line detaches a block;
`open` lines, edges and applications take nothing; a whitespace-only comment is no
definition; claiming both forms at once is `E_DEFINITION`, as is prose that fails
the definition gate — located diagnostics, all of a file's in one pass. The
attachment claim and the prose gate are owned by `definitions-parse-from-comments`,
`definitions-are-identity-prose` and `obligations-never-define`; the surface rule
here is only that comments are the syntax those cards ride on.

## System Context

Files are edited by humans and agents side by side, and the diff is the change
record (`source-is-the-only-truth`): a line-oriented, indentation-structured syntax
keeps every semantic change a clean line diff. The offside rule follows the lexer's
token stream — `NL`, `INDENT`, `DEDENT` — so the grammar itself stays context-free
(`the-grammar-fits-on-one-page`).

## Satisfy

`Compiler.Lexer` (the offside-rule tokenizer: tracks the indentation stack, emits
`INDENT`/`DEDENT` pairs, rejects tabs in indentation and stray dedents, and
captures every comment with its position instead of discarding it).

- test — lexer::blocks_produce_indent_dedent, lexer::nested_dedents_unwind_fully, lexer::eof_closes_open_blocks
- test — lexer::blank_and_comment_lines_are_invisible
- test — lexer::tabs_in_indentation_are_rejected, lexer::stray_unindent_is_rejected
- test — lexer::comments_are_captured_with_lines_and_positions
- test — parser::reserved_words_cannot_name_elements
