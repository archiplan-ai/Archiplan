# Verdict quality — confidence calibration & feedback

Two properties that make verdicts **trustworthy** and make the analyzer
**improve over time**. They are independent of the analytics layer
([`episodes.md`](episodes.md), [`fault-trees.md`](fault-trees.md)) — they raise
the floor of every verdict.

## Evidence sufficiency / confidence calibration

A verdict's confidence must reflect **how much the run actually verified** — not
the model's self-assurance:

- which sources it reached versus which it could not (a degraded fetch lowers
  confidence);
- whether the root cause is **cited to concrete evidence** or merely inferred;
- a verdict that could not reach the evidence it needed must **say so**, at a
  confidence that reflects the gap.

This makes a low-confidence verdict an explicit, actionable state rather than a
silent guess.

## Feedback loop

The analyzer must improve from operator judgment:

- an operator **confirms or corrects** a verdict (wrong classification,
  accepted / rejected fix);
- that signal is captured and fed back to tune **deduplication identity** (so a
  mis-split or over-merge is corrected — the identity-drift question in
  [`incident-lifecycle.md`](incident-lifecycle.md)) and **classification** (so
  labels improve);
- feedback is **durable and attributable**.

Without this loop the analyzer is static; with it, every operator correction is a
training signal.

## Requirement

Every verdict must carry an **evidence-sufficiency-grounded confidence**; and
operator **confirmation/correction** of verdicts and fixes must be captured and
fed back to improve deduplication identity and classification.

## Questions

- The confidence scale, and how evidence sufficiency maps onto it.
- Whether feedback adjusts behavior automatically or via operator-reviewed updates.
- Attributing a fix's success (did the episode stop recurring after the fix
  shipped?) as implicit feedback.
