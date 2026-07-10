---
kind: non-functional
origin: intent
satisfied-by: [Archive]
deferred:
---

# Keyframes bound the archive

The first version is a keyframe: the full canonical render. A later save writes a
keyframe exactly when the patches since the last keyframe — including the one this save
would write — together outgrow the new render; otherwise it writes a unified diff against
the previous version's canonical bytes. Total archive bytes therefore stay within about
twice the keyframe bytes, whatever the churn pattern. Patches apply mechanically to
hash-verified input — their three lines of context serve the human reader, not fuzzy
matching — and stay reviewable on purpose: the patch is the round's permanent change
record, while keyframes are marked generated so forges collapse them in review.

## System Context

In a git repository, compression is canonicalization plus plain text: compressed binaries
would cost their full size in history forever and refuse diff and merge; full snapshots
would put a model-sized file in every review. The manifest's `kind` field keeps encodings
per-entry, so the policy can evolve without rewriting history.

## Satisfy

`Archive` (the keyframe policy at save; reconstruction walks the nearest keyframe plus
forward patches and verifies the seal).

- test — versions::first_save_keyframes_then_patches_and_reconstructs
- test — versions::a_rewrite_outgrows_its_patch_and_keyframes
