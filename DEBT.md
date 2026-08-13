# Debt

Weaknesses seen during work and not yet pressed. Each entry names where it lives, what it
costs, and what a stress round would have to decide. Nothing here is a decision. An entry
leaves this file when a stressor carries it, and the stressor's resolution says so.

Seen 2026-08-11, during the unit `the-workaround-is-the-record`.

---

## One lexical test decides both what is proposed and what is demanded

`crates/archi/src/links/capture.rs:426-434`. For every changed file-symbol, every task that
claims the file, and every spec ref of that task, one boolean decides two separate things:

- whether a candidate link is minted, and
- whether the ref enters `pressed`, which is what the wave gate demands coverage for.

The boolean is `ref_terms(spec_ref)` and `item_terms(file, content)` sharing at least one
word. `crates/archi/src/plans/mod.rs:1549` reads `pressed` for the gate, and the test at
`crates/archi/src/plans/mod.rs:2644` states the consequence as intended behavior:
`unpressed refs never gap`.

The two directions do not cost the same.

- A word matches by accident. A wrong candidate is minted. The reviewer sees it at the gate
  and retires it; anything missed decays to confidence 0.00 and the audit asks for it again.
  Loud, and guarded twice.
- A word does not match although the code does realize the edge. No candidate is minted,
  **and the ref never enters the gate**. The wave closes reporting complete coverage,
  having never asked. Silent, and guarded by nothing.

Measured on this unit: nine candidates offered for the two refs the gate held, of which
seven were anchored at a file that realizes neither. 48 of the 229 decayed rows now standing
in the journal are anchored at one file this effort wrote. That is the loud direction, and
its size says how coarse the test is.

What a round would have to decide: whether one signal can carry both jobs, or whether
proposing a candidate and demanding coverage need separate tests with separate thresholds.

## The no-signal list is a count, and the count is the whole surface

`crates/archi/src/links/capture.rs:609-611`. Pairs with no shared word are reported as
`suppressed <n> no-signal pair(s) — whole under --json`. This is the only place a
false negative can be seen, and this unit's wave printed `128`. A number that large is read
as noise and not opened, so the one surface over the silent failure is closed by its own
size.

What a round would have to decide: whether the list needs narrowing to be worth printing —
by ref, by task, by whether the ref is otherwise uncovered — or whether the gate should
speak for itself instead of relying on a reader.

## Refusals that do not name the continuation

`refusals-name-the-continuation` is a standing requirement. Two refusals break it.

The third was `archi link confirm l2748` answering ``no live link `l2748` `` while the row
stood and was listed, because the address needed its digest. It went with the verb when
`a-link-stands-asserted-or-it-does-not-stand` retired the second standing.

- `archi plan task req remove` does not exist as a verb, and a refusal elsewhere names it as
  the way forward. Seen earlier in this effort; the way around it was to hand-edit `owns:`.
- `archi plan verify` on a record whose bullet wrapped onto a second line answers
  ``line <N>: stack bullets are `- <detail>` ``, and the same shape for architecture
  bullets. The rule broken is that a bullet occupies one line; the line it points at is the
  continuation, which carries no `- ` and therefore looks like prose in the wrong place. The
  refusal restates the grammar and never says the bullet wrapped, so the reader re-reads a
  bullet that is already correct except for its width. Cost two rounds of guessing on this
  unit alone.

Both leave the reader without the next move, which is the whole of what that
requirement asks for.

What a round would have to decide: nothing about design — these are defects against a
requirement already standing. They need a task, not a verdict.

## A closed plan holds a requirement it will never build

`crates/archi/src/docs/mint.rs:129-141`. `req_rm` walks `all_plans(root)` and refuses while
any task anywhere names the slug in `owns:`. It reads no plan state — a completed plan holds
the slug exactly as a draft one does. So once any plan has ever owned a requirement, that
requirement can only be retired by editing the record of work that already finished.

Seen on `the-briefing-says-what-help-does-not`: four holders, `quiet-the-world`,
`scenario-shape`, `world-layers` and `the-workaround-is-the-record` — all completed, none
building anything.

What the refusal is protecting against is real for a live plan: retire a slug a wave is
about to prove and the wave loses its contract. A closed plan has no contract left. Its
`owns:` is a record of what it owned while it ran, and a record that must be rewritten
before the tree can move is not a record.

Measured, the hold is also nearly empty. Deleting the file with the holders left alone
leaves `archi check` at exit 0 with no finding; only `archi plan verify`, pointed by hand at
one of those closed plans, reports `owns ... which the reverse lookup does not match —
structurally broken`. No flow runs `plan verify` on a completed plan. So the refusal blocks
a safe removal on the strength of a check nobody performs.

The operator's call on this tree was to delete the requirement and leave the four plans as
they stand.

What a round would have to decide: whether a plan's hold on a slug ends when the plan
closes, and if it does, what `plan verify` should say when it is aimed at a closed plan that
names a retired slug — a note about history, or nothing at all.

## The test fixture may not survive being run in parallel

`crates/archi/tests/util/mod.rs:105`. Reported by the cleanup sweep of this unit and **not
reproduced**: one `cargo test` run out of five failed five `world_e2e` cases, each at
`git ["commit", "-qm", "seed"]` in the temp-repo setup, with empty stderr. Four later runs
of the same command were green, and the full suite has been green on every run since.

This is hearsay against a flake, recorded because a fixture that fails one run in five will
eventually fail a run somebody trusts, and an empty stderr gives that person nothing. It is
not attributed to this unit: the setup predates it.

What a round would have to decide: nothing yet. Somebody has to see it a second time and
capture the stderr before there is anything to press.

## An anchored test body cannot be tidied without moving the anchor

`crates/archi/src/docs/world_check.rs`, lines 1398, 1423, 1448, 1919, 1961 and 2002. The
same expression stands inline six times:

    .map(|d| (d.code, d.file.as_str(), d.line)).collect::<Vec<_>>()

A named helper for it now exists at line 1785, written by a later wave and used by that
wave's two tests alone. Folding the other six is the obvious tidy, and it compiles. It was
not done, because each of the six sits **inside** a test body that a code-link anchors, and
the duplication is inline rather than callable — so the fold *is* a body move. A single-site
probe added seven `body moved; the watched interface holds` rows to `link verify`. The house
rule set in commit `997b120` is that a fold keeps anchored bodies still, using thin wrappers
where it must; there is no wrapper shape available here.

So the rule and the tidy are in direct conflict, and the rule wins by default every time,
which means this duplication is not going to be removed by any ordinary sweep. That is the
part worth pressing: whether an anchored body is genuinely frozen, or whether a repin is the
cheap answer and the sweeps have been avoiding it for no reason.

What a round would have to decide: what a code-link is anchoring to — a body that must not
move, or an interface that a repin is allowed to follow.

---

Related: `archi/world/notes/the-words-in-the-code-are-not-the-words-in-the-design.md` — why
the silent direction is expected to be worse on any project that is not a tool modeling
itself.
