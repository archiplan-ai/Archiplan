---
name: archi-search
description: How to find anything in an archiplan project — the semantic menu first, then the structural reads from an element, ranked search last, grep never. Read this page before you hunt for nodes, claims, conditions or files. Every archi workflow skill points here.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-search/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

# Archi search — how to find anything

One order, top down. Start at the top, and drop a level only when the
level above cannot answer.

1. **`archi query --top` — the semantic menu.** Every node with its
   identity sentence: pick by meaning, not by guessing names. `--json`
   is the shape it comes in; pipe it into `archi viz` to draw a slice.

2. **From an element, three structural reads answer exactly:**
   `archi req ls --satisfies <element>` — the requirements that name it
   (similar and contradicting claims live in one cluster);
   `archi world ls --covers <element>` — the outside conditions on it;
   `archi link ls --spec <ref>` — the files recorded against an element,
   a requirement (`req:<slug>`) or a scenario (`<fact>#<scenario>`).

3. **`archi search <phrase> [--kind ...]` — last**, when you hold a
   phrase and no address. Hits come ranked, and each hit carries its
   addresses, so the next command starts there.

**Search, do not grep.** Grep misses the model, because definitions live
in the compiled graph and not on disk as prose. Narrow the search with
`--kind`. Machine-read it with `--json`.

The order is measured, not taste: on this very tree, lexical search
found 10 of the 31 requirements standing on one element — two thirds of
the claims never say the element's name in prose. The structural read
found all 31.
