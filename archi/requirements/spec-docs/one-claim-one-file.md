---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# One claim, one file

A requirement is a unit of desirable shape — a claim the architecture must uphold — and
it lives at one of three scales wearing one schema: a section (a heading inside its
parent's file — name plus prose, inheriting every field, satisfaction included); a file
(`<slug>.md` with its own frontmatter); a folder (`<slug>/` with the folder-named file
inside, for a claim whose subrequirements outgrew inline sections). Containment is the
hierarchy: files at an intent folder's root are that intent's requirements, files in an
epic's folder are its subrequirements — the path is the parent pointer, and there is no
back-reference to drift. Promotion between scales is mechanical and changes no meaning.
Within a file, `System Context` and `Satisfy` are reserved; any other heading opens a
subrequirement — headings are structure, not decoration.

## System Context

A search hit must be the whole story — claim, context, satisfaction, proof — with the
name and summary as the card (`cards-carry-the-next-hop` renders them); one claim per
document node is what keeps a retrieval window from ever splitting a field.

## Satisfy

`DocsCompiler` (walks intent folders into requirement trees, headings into
subrequirements, and derives each parent from the path alone).

- test — docs::the_worked_tree_checks_out
- test — md::structure_parses
