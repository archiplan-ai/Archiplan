---
affects: [WorldDoc, DocsCompiler]
outcome: surviving
---

# The rename moves the model while the world stands still

Rename a node in `.arch` during an ordinary refactor. Every world fact whose `covers`
names the old path stops resolving, and `check` blocks until each one is edited.
Nothing about the world moved: the operator still learns of the collision late, the
condition is word for word what it was. The world built to outlive the design breaks
on the first change to the design.

## Attractor

`covers` becomes the field an operator leaves empty to keep `check` green. Every
empty `covers` is also a `world_orphan`, so the finding that marks dead weight starts
firing on the healthiest facts in the tree. The bridge from a node to what conditions
it — the one path that lexical retrieval cannot replace — decays into a formality.

## Resolution

Held. `satisfied-by` resolves against the live model in exactly the same way, and the
repository has carried that contract since the first version: a rename edits every
requirement that names the moved path. The world inherits the contract instead of
inventing a second one, so the pressure finds no new failure here. What it does find
belongs to the model-wide question of renames, not to this world.
