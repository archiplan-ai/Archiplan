---
kind: functional
origin: intent
satisfied-by: [DocMint, Cli]
deferred:
---

# One verb mints the world fact

`archi world add "<title>"` writes one file under `archi/world/` in the exact schema
shape — the frontmatter keys present and empty, the headings in order, the text slots
empty — and a person writes the prose. `archi world rm <slug>` retires one fact, and
it refuses while another fact names that slug in `uses`, naming the dependants. The
slug comes from the title, as it does for a requirement and a stressor.

## System Context

Requirements and stressors already work this way: the command makes the skeleton and
the machine fields, the person writes the text, and `check` holds the empty slots
until the prose lands. The world wing joins that convention rather than inventing a
second one, so an operator who knows `req add` needs nothing new. A refusal that
names the dependants is what makes deletion safe: the `uses` chain is exactly the
blast radius, and the operator sees it before the file goes.

## Satisfy

`DocMint` (mints the world skeleton beside the requirement and stressor skeletons,
derives the slug and the placement from the tree, and pre-flights the removal against
the inverse of `uses`). `Cli` (the `world` verb: `add` and `rm`, with a missing
parameter refused and the exit codes of the existing doc verbs).

- test — `world add` writes the three frontmatter keys empty and the headings in order
- test — a second `world add` with the same title refuses and names the standing file
- test — `world rm` on a fact that nothing names retires the file
- test — `world rm` on a fact named in another fact's `uses` refuses and lists the
  dependants
- test — the minted skeleton fails `check` until the prose lands, and passes after
