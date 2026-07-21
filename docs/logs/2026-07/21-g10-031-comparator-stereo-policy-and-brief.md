# g10.031 Comparator Stereo Policy And Brief

Date: 2026-07-21
Batch: 31.38
Status: complete; Batch 31.39 ready

## Result

The operator approved comparator-relative stereo promotion for creative
`Dream`. Local four-second mapped-window source balance and dominance remain
complete diagnostics, but finite values no longer reject by numeric threshold.
The same measurement now records Signal-source and PaulX-source values.

Hard stereo controls remain:

- finite deterministic exact-length output
- duplicate/mono, common polarity, anti-phase, swap, per-bin magnitude, and
  declared `space` behavior
- candidate-source whole-render and three-band balance within `0.75 dB`
- balance spread across `space=0`, `0.5`, and `1` within `0.50 dB`
- no whole/band dominance reversal above the frozen source floor

An eligible independent listener is terminal promotion authority for neutral
candidate/PaulX stereo and the `space` trios. The operator may reject on
speakers but cannot supply the independent pass.

## Fresh Candidate Authority

One complete `ComparatorAuditedRenewalSpectral` brief is frozen. It retains
the Batch 31.36 renderer formulas, admission seed, support audit, objective
gates, mono pack, linked phase law, exact boundaries, `32 MiB` state bound,
cleanup, and minimal admission. It adds no DSP repair. Low-frequency noise and
opposite entry/tail energy weighting remain explicit listening risks.

The fresh candidate must use worktree `signal-candidate-31-39`, branch
`candidate/g10-031-comparator-audited-renewal`, and a new private six-file
module. Deleted checkpoints and code remain rejected and may not be recovered.
No candidate DSP, test, harness, fixture, API, route, cache, Loophole, or
Chorus surface entered `main`.

## Validation

- `git diff --check`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`

All passed. `effigy doctor` retains the pre-existing god-file error and
attention-marker warning; this batch does not expand scope to address them.

## Next Task

Run Batch 31.39 only. Implement the frozen comparator-audited candidate once
from fresh source in its named disposable worktree. Stop at the first terminal
miss and delete it without repair or rerun. After a complete pass, retain the
isolated checkpoint and receipt for a separate admission batch. Do not merge
or push.
