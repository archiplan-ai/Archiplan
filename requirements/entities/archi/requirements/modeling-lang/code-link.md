# Code-links

spec ↔ code traceability

## What it is

**Code-links** are **authored** records that some **code** (a **`CodeRef`**: file, optional symbol path, content hash) **realizes** some **spec element** (a **`SpecRef`**: node id or typed edge, in a scope and version slot). They are **pinned** (hash + canonicalizer version) and **checked** with **`archi link verify`** when you want confidence the graph still matches the tree.