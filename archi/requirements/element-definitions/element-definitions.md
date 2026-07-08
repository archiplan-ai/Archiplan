# Element definitions

Graph definitions should be more semantically rich: every element can carry a free-text
definition. The comments already written in the spec files are the raw material — parse them
and validate them. A definition is a single sentence stating what the element *is*, at most
240 characters; multi-sentence prose and comma-spliced clauses using modal verbs (must,
should, shall, ensures, handles) are rejected. Obligations go into requirement docs, not into
definitions — constrained identity prose, not an open text field.
