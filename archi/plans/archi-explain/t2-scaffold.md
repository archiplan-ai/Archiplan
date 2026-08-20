---
node: Scaffold
owns: [the-why-reads-back-from-the-record]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t2 — Scaffold

the archi-explain page under the new api

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi-explain.md
- skills/archi.md
- crates/archi/src/scaffold.rs
- crates/archi/tests/init_e2e.rs

## Stack

- `skills/archi-explain.md` is new: the freshness header naming
  `.claude/skills/archi-explain/SKILL.md`, the `archi-search` pointer line (resolving a
  phrase to an address is that page's job, not this one's), the read-only mandate, then
  the chain in order — `world ls --covers` (the condition the behavior serves),
  `req ls --satisfies` (what must hold; each file's `origin:` names its stressor),
  `decision ls --links` (the recorded trades; t1 builds it this same wave, the flags
  above are the contract), the stressor files (description, attractor, verdict,
  Resolution), `version list` and `version diff <a> <b>` (the tree never moves — no
  checkout dance), `link ls --spec` (who realizes it today) — then the answer rules from
  the old page, kept in spirit: lead with the answer; quote decisions verbatim and name
  the alternatives that lost; cite ids and origins; surface the pressure trail; silence
  is a real answer — no decision means no recorded trade-off, say so and offer to record
  one; never invent rationale
- `skills/archi.md`: the "Show, do not tell" ground rule gains one sentence — the
  structure half is `query | viz`, the why half is the `archi-explain` skill
- `scaffold.rs`: `SKILLS` nine -> ten; `init_e2e.rs`: the install count 13 -> 14, the
  `EMBEDDED_SKILLS` table gains the row (the pointer and confirm-candidates guards then
  read the new page automatically), plus the content guards from the verify bullets
- the doctrine-singleton guard stands: the page names commands in its chain steps but
  must not reproduce `archi-search`'s distinctive sentences

## Verifications

### the-why-reads-back-from-the-record

- test — a fresh init installs `archi-explain` byte-equal to the embedded copy
- test — the embedded page orders the chain: world, then requirements, then decisions,
  then stressors, then versions, then links
- test — the page says silence is a real answer and forbids invented rationale
- test — the page is read-only in as many words and mutates nothing
