---
affects: [DocMint, Search]
outcome: surviving
---

# Slug namespaces collide in retrieval

Nothing stops a requirement, a stressor and an intent from sharing one slug: the
mints check uniqueness per kind. Retrieval by bare slug then answers three cards.

## Attractor

Slugs stop being addresses; every cross-reference needs a kind qualifier and every
agent guesses.

## Resolution

Held — the references that matter are already kind-scoped: `origin: stressor(…)`
resolves in the stressor namespace, `satisfied-by` in the model, plan `owns` in
requirements; search cards carry their kind and `--kind` narrows. A global slug
namespace would buy cosmetic uniqueness at the price of renaming half the tree.
