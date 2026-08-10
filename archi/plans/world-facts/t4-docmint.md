---
node: DocMint
owns: [one-verb-mints-the-world-fact, removal-names-the-code-it-strands, retirement-refuses-a-plan-in-flight, the-refusal-is-an-ordered-continuation, the-wing-arrives-without-noise]
---

# t4 — DocMint

mint and retire a fact, pre-flight what stands on it

## Spec

- `DocMint`
- `Function type_of DocMint`
- `Cli.drive consult(->Command, <-Report) DocMint.mint`
- `Cli.drive consult(->Command, <-Report) DocMint.remove`

## Inputs

- from t1 — the schema shape the skeleton must emit and the slug rule

## Outputs

- crates/archi/src/docs/mint.rs

## Stack

- the skeleton path is `archi/world/<slug>.md`, created with its parent folder
- the removal reads the folded link set and the plan records beside the `uses` inverse
- the refusal renders one ordered command list, reusing the refusal helper the other doc verbs use

## Verifications

### one-verb-mints-the-world-fact

- test — world_e2e: `world add` writes the three frontmatter keys empty and the headings in order
- test — world_e2e: a second `world add` with the same title refuses and names the standing file
- test — world_e2e: the minted skeleton fails check until the prose lands, and passes after

### removal-names-the-code-it-strands

- test — world_e2e: `world rm` on a fact whose scenario a link names refuses and lists the ids
- test — world_e2e: the refusal names the code item of each stranded link
- test — world_e2e: after `link rm --spec` the same `world rm` proceeds

### retirement-refuses-a-plan-in-flight

- test — world_e2e: `world rm` refuses while an open plan's task carries the fact, naming plan and task
- test — world_e2e: the same fact retires once that plan is completed

### the-refusal-is-an-ordered-continuation

- test — world_e2e: a fact blocked by a plan, a link and a dependant prints three ordered commands
- test — world_e2e: each printed command runs as written and clears its blocker

### the-wing-arrives-without-noise

- test — world_e2e: `world add` on a tree with no `archi/world/` creates the folder and writes the file
- test — check_e2e: a tree with no `archi/world/` passes check with no world finding
