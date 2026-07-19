# g10.031 Cyclic Owner Rejection

Date: 2026-07-19
Status: Batch 31.11 complete; candidate rejected

## Result

Implemented the frozen `CyclicGrain` brief once in the disposable
`signal-candidate-31-11` worktree on
`candidate/g10-031-cyclic-grain`.

The private candidate contained only `creative_cyclic/{mod,plan,schedule,grain,
synthesis,tests}.rs` and one private `lib.rs` declaration. It used the frozen
sample-centred map, deterministic lattice, at most two unit-rate reads,
normalized raised-cosine crossfade, linked-channel scheduling, bounded
mid/side `space`, exact target crop, and duration-independent rolling state.

All seven structural tests passed:

- request rejection and byte-exact identity
- exact length, finiteness, silence, and repeatability
- monotonic map, macro geometry, and at-most-two-grain scheduling
- seed-controlled shared lattice phase
- impulse energy confined to scheduled read support
- duplicate, swap, polarity, and width mechanics
- peak bounds, normalization coverage, `8 MiB` state cap, and unchanged
  short/long capacities

## Stop

Creative synthetic admission failed on its first neutral row:

- source tone: `110 Hz`
- ratio: `2x`
- measured dominant frequency: `111.328 Hz`
- pitch error: `20.778` cents
- frozen ceiling: `15` cents

The dominant cause is pitch displacement from crossfading source-offset
unit-rate grains. The miss is `5.778` cents. The frozen rule permits no
grain-length, hop, window, interpolation, seed, threshold, or test-tone sweep.

No later synthetic row, comparator capture, long-form mono render, `16x`
probe, or linked-stereo listening ran.

## Cleanup

Deleted the candidate worktree, branch, private module, tests, and local build
state. No candidate DSP, harness, fixture, report mode, generated audio, public
API, cache, route, Loophole, or Chorus surface entered `main`. The three
unrelated binaural/reverb edits remain untouched.

## Validation

- candidate compile-only gate: passed
- candidate structural filter: `7/7` passed
- candidate creative synthetic filter: failed on the first frozen pitch row
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- `effigy doctor`: unchanged pre-existing god-file and attention-marker
  findings
- disposable worktree and candidate branch: absent

## Next Task

Execute `g10.031` Batch 31.12 only. Reassess cyclic ownership at architecture
level or close the explicit character. Do not tune or reimplement
`CyclicGrain`.
