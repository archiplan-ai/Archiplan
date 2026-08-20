> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-migrate-links/SKILL.md`. When the act is `updated`
> or `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

> Retrieval — how to find anything here — is the `archi-search` skill.

# Migrate a journal written by the matcher

A project modeled before the declaration carries a journal capture wrote by
itself. That capture minted a link wherever the name of a changed file shared
one word with the name of a model element, and it minted one per symbol. The
rule is gone from the tool. The rows it wrote are still in the journal, still
graded on every verify, and still counted as coverage.

This skill is the one pass that sorts them: what a person stood behind stays,
what the matcher guessed and nobody ever read is retired, and the cost of the
retirement is measured before it is paid.

Run it once per project. A journal that has already been through it reports
zero inferred rows and there is nothing to do.

## Ground rules

**The journal is append-only and this skill does not break that.** A retire is
an event like any other: the `add` stays, the `retire` lands on top, and the
fold shows the row gone. Nothing is rewritten and nothing is deleted from the
file. Never open `archi/links/journal.jsonl` in an editor — every move here is
a verb.

**Use the binary of the checkout you are in.** In a worktree, `./target/release/archi`.
The `archi` on `PATH` is often a symlink into a different checkout, and it will
answer about that project's journal instead of this one. Getting this wrong
produces a confident report about the wrong tree.

**Mutations need a worktree.** `archi worktree mint migrate-links`, then `cd`
there. The retire is a large, single, reviewable commit and it belongs on a
branch like any other work.

## 1. Read the shape before you touch anything

Every row carries the rule that produced it. The listing is
`id · kind · standing · origin · rule · <spec ref> ← <anchor>`, so the rule is
the fifth column:

```sh
archi link ls | awk '{print $5}' | sort | uniq -c
```

Three words appear. `authored` is a person typing `archi link add`. `declared`
is a task's writer naming its own work. `inferred` is the matcher. Only the
third is this skill's subject; the other two are somebody's claim and they
stay.

When `inferred` is zero, stop. There is nothing to migrate.

## 2. Measure the redundancy — this is the decision

The matcher minted one row per symbol, so one claim about one file arrives
dozens of times over. Count the distinct claims the inferred rows make against
the number of rows:

```sh
archi link ls | awk '$5=="inferred"' | wc -l
archi link ls | awk '$5=="inferred"' | sed 's/ ←.*//' \
  | awk '{$1=$2=$3=$4=$5=""; print}' | sed 's/^ *//' | sort -u | wc -l
```

A ratio near one means the matcher was accurate on this tree and the rows are
worth reading one by one. A ratio of ten or more means the corpus is one claim
repeated, and reading it row by row is not a review — it is the wall that
trains a reader to skim. On the project this skill was written from, 2141
inferred rows carried 102 distinct claims: twenty-one anchors each, and one
claim wore seventy-two.

Then measure what the retirement would cost, which is coverage — the count of
distinct spec refs any live link answers:

```sh
archi link ls | sed 's/ ←.*//' | awk '{$1=$2=$3=$4=$5=""; print}' \
  | sed 's/^ *//' | sort -u | wc -l
```

Write the number down. Run it again after step 4 and report both. On the
project this was written from it went 159 to 149: ten refs lost, and every one
of them was a ref whose only evidence was a shared word.

## 3. Read the claims, not the rows

Take the distinct claims from step 2 and read that list — tens of lines, not
thousands. For each one ask the only question that matters: **does this file
answer this part of the model?** You are not checking whether the anchor
resolves. `link verify` already knows that. You are checking whether the pair
was ever true.

A claim worth keeping is re-authored rather than rescued. Write it again:

```sh
archi link add "<spec ref>" <file>#<symbol> --kind indirect
```

That gives it one anchor chosen on purpose instead of twenty chosen by a word,
and the new row reads `authored`, which is what it now is. Do this before the
retire, so `link verify` never passes through a state where the claim is
missing.

**Put the ones you keep in the commit message.** A reader a year from now needs
to know which claims survived a mass retire and why, and the journal records
only that they were minted.

## 4. Retire the rest

Collect the ids and retire them in batches. `link rm` takes many ids at once
but a shell has an argument limit, and **zsh does not word-split an unquoted
variable**, so a naive `link rm $ids` sends the whole list as one id and fails
with a confusing message:

```sh
archi link ls | awk '$5=="inferred" {print $1}' > /tmp/inferred.txt
xargs -n 150 ./target/release/archi link rm < /tmp/inferred.txt
```

Re-run step 1. `inferred` is now zero, or holds only the rows you deliberately
kept.

## 5. Verify, and repair what the retire uncovered

```sh
archi link verify
archi link audit
```

Three things surface here and each has one move:

- **`missing`** — the anchor names a symbol that is not in the tree. The
  matcher anchored on test bodies and helper functions, and those get renamed.
  `archi link repin <id> --to <file>#<symbol>` when the code moved and the
  claim stands; `archi link rm <id>` when the symbol is gone for good.
- **`drifted`** — the body moved under a claim that still holds.
  `archi link repin <id>` re-pins it. Repin only what this pass touched. Rows
  that arrived drifted from somebody else's work are not yours to sign.
- **`unlinked spec element`** in the audit — a part of the model that now has
  no code against it. Some of these are the ten refs the retire cost. That is
  the honest state: the coverage was a word match, and it is better read as a
  gap than as an answer.

## 6. Commit as one unit, and report

One commit. The message carries the four numbers — rows retired, distinct
claims they expressed, coverage before, coverage after — and the list of
claims you re-authored by hand. Then land the worktree with the
`archi-finish-worktree` skill.

Report to the operator in those same numbers. "Retired 2141 rows expressing
102 claims; coverage 159 to 149" is a sentence they can argue with. "Cleaned up
the links" is not.

## Principles

- **The rule word is the whole triage.** `authored` and `declared` are
  somebody's claim. `inferred` is a guess nobody read.
- **Read claims, not rows.** The matcher's corpus is one claim repeated, and a
  row-by-row review of it teaches skimming.
- **Measure the cost before paying it.** Coverage before and after is the
  price of the retire, in one number.
- **Re-author what you keep.** A claim worth keeping deserves an anchor chosen
  on purpose, and the `authored` rule that says a person chose it.
- **Retiring appends.** Nothing is rewritten, nothing is lost, and the record
  of what the tool once believed stays readable.
