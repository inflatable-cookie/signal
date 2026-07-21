# g10.031 Comparator-Audited Renewal Rejection

Date: 2026-07-21
Batch: 31.39
Status: complete; candidate rejected at synthetic admission

## Evidence

- worktree: `signal-candidate-31-39`
- branch: `candidate/g10-031-comparator-audited-renewal`
- immutable checkpoint: `c0cd943f5a5e8499540d5e759aac7a1586579d0a`
- compile: pass
- construction: exactly `1/1`
- structural: exactly `15/15`
- synthetic: exactly nine selected; seven passed, `Y04` and `Y09` failed
- mono and stereo listening: not run

`Y04` found two active regions in the `16x` impulse row. The secondary was
`-29.801787859 dB`. Batch 31.40 corrected the interpretation: `-30 dB` is the
active-window threshold, not a secondary-peak ceiling. The second active
region fails the frozen one-region / `None` requirement.

`Y09` failed linked-stereo swap at `4x` and `8x`. The runner had already
started all nine owners, so this second terminal result completed after the
first failure had begun cancellation. No gate was rerun.

## Decision And Cleanup

Objective admission rejected the complete candidate. No mono pack, stereo
assembly, speaker pre-screen, or independent review opened. The candidate was
not tuned or repaired.

Deleted the disposable worktree, branch, checkpoint reference, private
six-file module, tests, and build state. The checkpoint is no longer
branch-reachable. No DSP, test, harness, fixture, API, cache, route, Loophole,
or Chorus surface entered `main`. Nothing was pushed.

Batch 31.36 passed `Y04`, `Y09`, and the full synthetic gate under the
nominally same frozen renderer formulas and `ADMISSION_SEED`. Batch 31.39's
`7/9` result therefore cannot honestly authorize a parameter repair or another
implementation. The authority and executable evidence construction must be
reconciled first without recovering either deleted checkpoint.

Repo posture remains `baseline-routing`; no strict lane is active. Batch
31.40 is ready as docs-only reproducibility and evidence-authority
reassessment.

## Validation

- `git diff --check`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`

All passed. `effigy doctor` retains the pre-existing god-file and
attention-marker findings; this batch did not expand scope to address them.

## Next Task

Run Batch 31.40 only. Reconcile why Batch 31.36 passed `Y04` and `Y09` while
Batch 31.39 failed them under the nominally same frozen renderer and seed.
Either restore one executable, reproducible authority without implementing DSP
or close the renewed candidate path. Do not recover deleted code, change
thresholds, rerun candidates, open other characters or routing, touch Loophole
or Chorus, or push.
