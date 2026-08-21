---
node: Scaffold
owns: [each-node-names-the-condition-that-needs-it]
facts: [what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1]
---

# t1 — Scaffold

the node question at the three walks

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`
- `AgentBrief`
- `Scaffold.emit persist(AgentBrief) SourceTree.store`

## Inputs

## Outputs

- skills/archi.md
- skills/archi-migrate.md
- crates/archi/tests/init_e2e.rs

## Stack

- the canonical sentence, word-identical everywhere it appears:
  "which outside condition stops holding if this node is gone?"
- placement 1 — `skills/archi.md` step 3 (Capture the world), a short paragraph after
  "Write the fact from what the operator already told you": the spoken task under-reaches —
  the condition a layer exists for goes unsaid because it is obvious; ask the question of
  every node a fact will cover; draft the answer first and put it through the poll tool
  (`AskUserQuestion` in Claude Code, the equivalent elsewhere), the drafted condition as an
  option, another shape of it as a second, and "nothing outside reaches this node" last,
  routing to `.worldignore`; a symptom cannot answer, because the symptom survives the
  node's removal
- placement 2 — `skills/archi.md` step 5 (Draft the model), one or two sentences where
  elements land: as each node lands, ask the question once, draft and poll the same way;
  an answer standing is the next fact to write, no outside condition is the node's
  `.worldignore` line
- placement 3 — `skills/archi-migrate.md`, the world pass, after the drafts step: walk the
  covered nodes once with the same question, drafted and polled like the pass's other
  questions; a node whose only covering fact is symptom-shaped has its real fact still
  unwritten — the question mints the candidate the prose never carried
- guards in `crates/archi/tests/init_e2e.rs`, three per the verify bullets; the identity
  guard extracts the sentence from each page and asserts byte-equality across the three
  placements; the standing capture/world guards keep their assertions untouched
- `scaffold.rs` untouched: both pages are embedded by path

## Verifications

### each-node-names-the-condition-that-needs-it

- test — the question sentence stands word-identical in the workflow skill's capture and
  model steps and in the migrate page's world pass
- test — each placement drafts the answer and puts it through the poll tool, with the
  no-outside-condition option routing to `.worldignore`
- test — the capture placement says a symptom cannot answer because it survives the node's
  removal
