---
kind: functional
origin: intent
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# The header points three ways

The frontmatter carries exactly three lists, and each points at a different kind of
target. `covers` names model elements by absolute path — the nodes whose behavior
this fact conditions. `sources` names the raw material the fact rests on, by path
into the tree. `uses` names other world facts by slug, and it means precondition:
this fact holds only while the named fact holds. Every entry resolves, and an entry
that does not is an error. Confidence is no field of its own: an empty `sources` is
the hypothesis state.

## System Context

The three fields are the whole coupling of the wing — down into the model, out into
the raw material, sideways into the other facts — and each one is checkable, which
is why there is no fourth. `covers` is also the only reliable bridge from the
vocabulary of the world to the vocabulary of the model: a fact is written without
the nouns of the model, so lexical retrieval cannot cross that gap and the traversal
goes structurally, from the node to the facts that cover it. Only one direction of
`uses` is stored, as `affects` and `satisfied-by` already are, and the compiler
expands the inverse.

## Satisfy

`WorldDoc` (the three lists as the whole frontmatter). `DocsCompiler` (resolves
`covers` against the live model, `sources` against the tree and `uses` against the
world slugs; an unresolved entry in any of the three is a located error; the inverse
of `uses` is expanded and never read from the file).

- test — a `covers` entry that names no model element raises a located error
- test — a `sources` entry that names no file in the tree raises a located error
- test — a `uses` entry that names no world fact raises a located error
- test — a fact with an empty `sources` passes and reads as a hypothesis
- test — a fourth key in the frontmatter raises a located error
