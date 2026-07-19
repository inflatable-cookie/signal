# g10.031 DiffuseSpectral Candidate Rejection

Date: 2026-07-19
Status: Batch 31.4 complete; candidate rejected
Roadmap: `g10.031`
Contract: `085`

## Result

Implemented the frozen `DiffuseSpectral` brief once in the disposable
`signal-candidate-31-4` worktree on
`candidate/g10-031-diffuse-spectral`.

The candidate contained only the private six-file `creative_diffuse` family
and one private `lib.rs` module declaration. It did not alter production
stretch tiers, public APIs, cache, routing, binaries, `Cloud`, or `Cyclic`.

Structural controls passed for exact length, finite output, complete rolling
normalization, exact silence, deterministic repetition, seed variation,
bounded duration-independent state, invalid requests, and linked-stereo
duplicate, swap, polarity, and width mechanics.

Creative synthetic admission then stopped on neutral `Dream` at `4x`.
Deterministic-noise crest-factor growth measured `7.08 dB`; the frozen ceiling
is `6 dB`. The measurement used active support. It was not caused by boundary
zeros or level matching. Completed pitch, impulse-replica, and non-periodicity
rows had passed.

## Decision

Reject the complete candidate. The dominant cause is uncontrolled stochastic
crest growth from the diffusive spectral field. The frozen topology permits no
limiter, compressor, crest repair, second normalization, or arbitrary level
change. A constant or seed sweep is not authorized.

No long-form or stereo-listening audio was produced. The worktree, branch,
candidate module, tests, and generated state were deleted. `main` remains
docs-only for this batch.

## Validation

- candidate structural filter: passed, `11` tests across the selected workspace
  filter
- candidate creative synthetic filter: failed on the frozen crest gate
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- disposable worktree and candidate branch: absent

## Next Task

Run Batch 31.5 only. Reassess crest ownership at architecture level and freeze
one complete replacement decision, or close `DiffuseSpectral`. Do not implement
a second candidate or open later creative owners.
