# g10.030 Event-Sealed Feasibility Rejection

Date: 2026-07-19
Batch: 30.5 and 30.6 family closure
Status: complete

## Start State

- `main` at `358ec19e`
- clean disposable worktree and
  `candidate/g10-030-batch-30-5-event-sealed` branch
- production renderer and retained harness unchanged

## Structural Stop

The frozen event refinement is:

`mean(E[n..n+16))-mean(E[n-16..n))`

For an isolated impulse at `e`, every `n` in `[e-15,e]` has the same maximum
score. The frozen earlier-sample tie break selects `e-15`. The frozen
structural gate requires the token at `e` exactly.

An exhaustive check over all `256` phases of the `H=256` lattice produced:

`phases=256 exact_failures=256 offset_min=-15 offset_max=-15`

This is an architecture contradiction, not a threshold miss. Fixing it would
change the refinement or tie rule and create the prohibited third
detector/window variant.

## Outcome

- stopped before renderer implementation
- generated no fixtures, reports, renders, or listening audio
- deleted the untouched worktree and branch
- retained production OfflineHighQuality byte-exact
- closed the multiresolution phase-vocoder successor family under Contract
  `084` Rule 7
- paused `g10.030` at the baseline-closure versus different-family intent
  checkpoint

## Validation

- `git diff --check`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`

## Next Task

Choose whether to close the stretch program on the frozen competitive baseline
or commission one complete successor from a non-phase-vocoder renderer family.
