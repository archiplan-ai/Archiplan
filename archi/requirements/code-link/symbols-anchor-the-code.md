---
kind: functional
origin: intent
satisfied-by: [Links.Canonizer, Links.Grader]
deferred:
---

# Symbols anchor the code

The resolution unit is the symbol: the item's path within its file — enclosing mod, impl
and trait names, then the item — which survives formatting, reordering and, moves being
candidate-tracked, file moves. Spans appear only in birth records: they say where code
was born, not where it lives. Non-symbol assets — configs, migrations, schemas — anchor
by file path. A projection carries two hashes over the anchored item, computed on the
canonicalized token stream rather than source bytes, so formatting and comment churn
never register: the interface hash over the declared shape — name, signature,
visibility — and the body hash over the whole item. Files outside the known languages
hash by whitespace-normalized text under the same contract.

## System Context

`drift-graded-per-kind` decides which of the two hashes a link watches, and
`hash-contract-is-versioned` stamps the canonicalizer into every pin — both stand on the
anchor defined here. Attributes and modifiers belong to their item; strings, comments and
lifetimes lex whole, so the canonical stream cannot split mid-token.

## Satisfy

`Links.Canonizer` (tokenizes, splits interface from body, hashes; the text fallback for
unknown languages). `Links.Grader` (resolves symbols through scope chains and file moves
before grading).

- test — code::a_fn_splits_interface_from_body
- test — code::scopes_nest_through_mod_impl_and_trait
- test — code::formatting_and_comments_never_move_hashes
- test — code::generic_text_files_hash_by_whitespace_runs
