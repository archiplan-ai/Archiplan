---
name: archi-explain
description: Answer why a thing is the way it is from the record — the world condition it serves, the claims that must hold, the recorded trades, the pressure behind them, the timeline, the code that realizes it. Read-only. Use when the user asks why an element exists, why it has its shape, or what happened to it over time.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-explain/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

> Retrieval — resolving the question's phrase to an address — is the
> `archi-search` skill. Come back holding an element or a slug.

# Archi explain — why it is the way it is

The user asks why an element exists, why it is split or merged, why it
has its shape, or what happened to it over time. Pull the answer from
the record — never speculate.

## Mandate

**Read-only.** The record answers: the world, the claims, the
decisions, the stressors, the versions, the links. Mutate nothing — not
the spec, not the plan, not the links, not the code, not the tests.

## Pull the explanation

Walk the chain in order, outside-in — the world first, the code last.

1. **`archi world ls --covers <element>`** — the outside condition the
   behavior serves. The deepest form of the question: not "which trade
   shaped this" but which condition makes the behavior necessary at
   all.
2. **`archi req ls --satisfies <element>`** — what must hold on it.
   Open the files: the claim is the summary, and each file's `origin:`
   names its stressor.
3. **`archi decision ls --links <name>`** — the recorded trades on an
   element or a doc slug: slug, `prefer → over`, the first phrase of
   the rationale. The file under `archi/decisions/` carries the rest.
4. **The stressor files.** `origin: stressor(<slug>)` is the address —
   `archi search <slug> --kind stressor` names the file. It holds the
   description, the attractor, the verdict in `outcome:`, and the
   Resolution.
5. **`archi version list`** — the timeline, one note per version — and
   **`archi version diff <a> <b>`** — the semantic delta between any
   two, `live` on either side — the tree never moves: the notes and
   the diff answer evolution, and nothing checks out.
6. **`archi link ls --spec <ref>`** — who realizes it today. `<ref>`
   is the element, or the claim as `req:<slug>`.

## Answer rules

Lead with the answer; back it with citations.

- **Quote decisions verbatim.** The title and the rationale in the
  decision's own words. Name the alternatives that lost: `prefer` is
  what won, `over` is what paid.
- **Cite slugs and origins inline.** `retry-budget (origin:
  stressor(burst-load)) requires …` — every citation carries its
  address.
- **Surface the pressure trail.** A claim born under pressure brings
  its stressor's description and verdict into the answer.
- **Silence is a real answer.** No decision linked means no recorded
  trade-off. Say "the record holds no rationale here" and offer to
  record one — a decision under `archi/decisions/` with `prefer` and
  `over`.
- **Never invent rationale.** When the record is silent, the answer is
  that it is silent — not a plausible story.

## Principles

- Decisions are the recorded why; the world is the deepest one.
- Requirements are the what; stressors are the pressure behind them.
- Versions are the timeline: the notes answer most evolution
  questions, the diff answers the rest.
- Silence is information — flag it, never fill it.
