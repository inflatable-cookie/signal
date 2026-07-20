# g10.031 Variance-Compensated Renewal Brief

Date: 2026-07-20
Status: Batch 31.22 complete; Batch 31.23 ready
Roadmap: `g10.031`
Contract: `085`

## Result

Froze `VarianceCompensatedRenewalSpectral` as fresh complete authority for the
still-untested PaulX-like neutral-`Dream` topology.

The renderer remains one exact sample-centred map, one sample-rate-normalized
long magnitude transform, deterministic frame/bin phase renewal, complementary
raised-cosine frame blending, variance compensation `1/sqrt(a^2+b^2)`, linked
mid/side ownership, deterministic exterior support, exact crop, bounded state,
and fixed-ratio offline execution at `4x`, `8x`, and `16x`.

No DSP law, reference row, source, threshold, listening rule, or minimal
admission surface changed. The Batch 31.21 implementation remains rejected and
deleted. Its compile miss supplied no acoustic evidence.

## Compile Boundary

Construction now has a complete private type and test surface. One clean
`effigy test compile` receipt and one local isolated checkpoint end
construction. Compiler-only plumbing may be repaired before that receipt only
when DSP and evidence semantics remain unchanged.

Structural admission and every later gate are one-shot from the checkpoint.
Any admission miss rejects and deletes the complete candidate without tuning
or rerun. The checkpoint is never pushed.

## Candidate Boundary

- worktree: `signal-candidate-31-23`
- branch: `candidate/g10-031-variance-compensated-renewal`
- private module:
  `crates/signal-dsp-stretch/src/creative_variance_compensated_renewal/`
- files: `mod.rs`, `plan.rs`, `analysis.rs`, `phase.rs`, `synthesis.rs`, and
  `tests.rs`, plus one private `lib.rs` declaration
- no public API, report mode, fixture, feature flag, dependency, cache, route,
  Loophole, or Chorus change

Authority:

- `docs/architecture/offline-creative-variance-compensated-renewal-spectral-brief.md`

## Validation

- candidate DSP and harness surfaces on `main`: absent
- rejected Batch 31.21 worktree and branch: absent
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- known `effigy doctor` findings: unchanged at 57 god-file and 5 attention
  markers

## Next Task

Run Batch 31.23 only. Implement the frozen brief once in its named disposable
worktree. Complete construction and compile validation before opening the
one-shot structural gate.
