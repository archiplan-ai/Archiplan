# Archiplan

Software hardening environment;

![](archiplan.svg)

Models are stored as source code: a project of `.arch` files — diffable, modular, compiled fresh on every run
(`requirements/modeling-lang/source-format.md`). The source is the only source of truth: the JSON statement layer
(`requirements/modeling-lang/modeling-lang.md`) is what it compiles to — and the read surface for agents — not a
second editing surface. `archi check` compiles and lints a project; `archi nkp` analyzes it;
`archi incidence` reads a stress round back as the stressor × component matrix and its findings
(`requirements/scoring/incidence.md`); `archi build --emit-batch` shows the lowered statements;
`archi link` ties spec elements to the code that realizes them and verifies the tie against the tree
(`requirements/code-link.md`).

## Workflow

intent ─→ requirements ─→ model ⇄ satisfy-claims ─→ version save ─┐
   ↑                                                              │
   └────── answer breaking stressors ←── stress session ←─────────┘   (harden loop)

hardened version ─→ plan use ─→ task add ─→ author plan.json ─→ start
   ─→ [ code ─→ plan next ─→ capture ─→ confirm ─→ gate ]  per wave
   ─→ scenarios ─→ DONE ─→ link verify / link audit          (execute loop)

1. Describe the intent. Write archi/requirements/<intent>/<intent>.md — a name and the problem statement, nothing else. It anchors the area; archi check enforces the placement rules around it.

2. Derive requirements. Files in that folder, one claim each: frontmatter (kind, origin: intent, empty satisfied-by, empty deferred) plus System Context and Satisfy. They're born open — check reports unsatisfied_requirement as an advisory finding, so open work is visible but never blocks. Epics are folders; refinements nest with origin: parent.

3. Draft the system model. .arch sources under src/; archi check compiles model and docs together. As the architecture takes shape, you fill each requirement's satisfied-by (which elements answer it — an architectural claim, made before any code) plus the Satisfy prose and verification bullets. From here a rename in the model breaks the build at the requirement that names it. archi nkp gives the landscape read while you shape it.

4. Harden through stress. archi version save -m … seals the render into the archive. Open a stress session pinned to that version, write stressors (affects validate against the pinned model, so the session stays coherent while you edit). Breaking stressors demand answers: new requirements with origin: stressor(…) and model edits. The next version save stamps the session closed and auto-fires the incidence report. Loop 2–4 until a round survives — that version is your hardened spec.

5. Cut the plan. archi plan use <name> pins the version you're at (refuses on a dirty model — hardening is the save). Then plan task add <node>, one per node: spec_refs seed as the node plus its incoming edges, and the requirements arrive by reverse lookup — you never retype them. Everything authored — envelope prose, inputs (which shape the waves), outputs (which scope capture), scenarios, and a verification per matched requirement — is a text edit of plan.json, with plan verify re-checking the whole file on every verb.

6. Execute in waves. plan start refuses until the structure is clean and every matched requirement carries a verification, then snapshots the tree (the item-hash index) and puts wave 1 in flight. You write code under each task's outputs; plan current-wave says what's open. plan next is the pivot: it captures the wave's delta — changed symbols in claimed files become candidate evidence links, pre-attributed by task — and then blocks on asserted coverage. You review (link ls --evidence), link confirm the load-bearing candidates, link rm the drive-bys (subtractions stick), and re-run plan next. The step that demands links is the step that just produced them. Unclaimed changes surface as leftovers rather than being guessed at.

7. Close. After the last wave, plan next prints the scenarios block (compose the end-to-end story), and one more plan next prints DONE. If the spec advanced mid-plan, plan repin moves the pin and plan verify shows exactly which tasks' obligations broke. plan close/plan reset are the manual overrides.

8. Live with it. archi check and archi link verify are the CI gates (drift graded per link kind); archi link audit is the hygiene sweep — dark deltas since the last version's commit, dark spec scoped automatically from the active plan's spec_refs, and evidence whose derived confidence fell below the floor (touches from later tasks raise it, unreconfirmed rewrites decay it, --prune retires the dead).

The through-line: each stage's output is the next stage's checked input — requirements name model elements the build verifies, stressors pin versions the archive reconstructs, tasks pin spec elements the plan verifies, and links pin symbols the verifier rehashes. Nothing is retyped, so every drift has exactly one place it can surface.