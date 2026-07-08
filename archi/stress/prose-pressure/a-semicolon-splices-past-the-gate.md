---
affects: [Compiler.Definitions]
outcome: breaking
---

# A semicolon splices past the gate

An author writes `def node Lexer: // offside-rule tokenizer; must reject tabs in indentation`.
The comma rule sees no comma; one semicolon carries the same obligation clause straight
through the gate.

## Attractor

The rule names one punctuation mark, and prose has half a dozen ways to join two clauses —
semicolon, em-dash, colon, parenthesis. A gate keyed to a token invites the token's synonyms,
and every synonym is idiomatic in this repository's own comment corpus already.

## Resolution

Broke exactly where the intent's literal wording drew the line: the rule is comma-keyed, the
smuggle is not. Enumerating splice punctuation is a losing game — the honest invariant is not
"no comma before a modal" but "no obligation in a definition", and the modal vocabulary itself
is the detection surface, splice or no splice. Answered by `obligations-never-define`: the
modal check drops its splice precondition entirely.
