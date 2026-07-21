# g10.031 Zero-Preserving Linked STN Reconciliation

Date: 2026-07-21
Batch: 31.50
Status: complete; zero-preserving v5 candidate ready

## Scope

Reconcile Batch 31.49's residual epsilon-log output with bit-exact silence.
Audit the complete mono, stereo, boundary, deterministic, memory, evidence,
and cleanup path. Change documentation only.

## Decision

Freeze `ZeroPreservingGeometryAuditedBoundedLinkedStnNoiseMorph`.

Its sole residual power interpolator returns positive zero when both endpoints
are exact zero. Every mixed zero/positive and positive/positive row retains
the v4 formula. Zero coherence, cross-power, mono and mid/side excitation,
mapped-envelope contribution, and final `f32` output are canonical positive
zero.

This is exact state ownership, not thresholding. No denoiser, silence fast
path, mask, duration state, variable traversal, stochastic change, extra
allocation, or post-render repair enters.

## Audit Result

- duplicate, common-negation, anti-phase, and swap laws retain zero exactly
- signed-zero input returns positive-zero silence
- locally zero mapped envelope contributes no residual bed
- `S12`, `S13`, `S15`, `S16`, and `S18` own the new rule
- structural and synthetic owner counts remain `18` and `10`
- transform, map, positive-power, source, comparator, listening, gate-order,
  receipt, cleanup, and minimal-admission rules remain unchanged
- `89 MiB` design, `96 MiB` actual, two-pass schedule, deterministic order,
  and fixed cost remain unchanged

## Repository Result

Fresh Batch 31.51 worktree, branch, module, prefixes, checkpoint, and cleanup
authority are frozen in the canonical brief. No DSP, test, harness, fixture,
dependency, API, route, cache, artifact, product, Loophole, or Chorus change
entered `main`. Pre-existing plugin edits remain untouched and unstaged.

## Next Task

Run Batch 31.51 only in the fresh v5 worktree and branch. Implement once,
complete compile and construction, freeze one checkpoint, then run structural
and synthetic admission in order. Stop before listening on any miss. Do not
recover Batch 31.49, merge, or push.
