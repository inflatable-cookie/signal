# g10.029 Oracle Adaptive Refocus

Date: 2026-07-12
Status: implementation ready

## Decision

Stop automatic-selector research. The passing time-adaptive painless transform
has not produced stretched audio, while Rényi, mixed-phase, calibrated
mixed-phase, and median-HPSS selection all rejected. Further detector work is
diminishing return without proof that ideal scheduling sounds better.

## Oracle Candidate

- manifest-declared transient centres; no detector in the render
- passing `512/1024/2048/4096` symmetric window islands
- absolute fixed-ratio source-to-output centre mapping
- current identity-locked phase policy generalized to actual variable hops
- exact output-side diagonal dual and guarded crop
- no phase reset, local unity stretch, component branch, crossfade, tail repair,
  stereo, dynamic ratio, or product route

Batch 29.6BA proves synthetic mechanism behavior and renders the existing 15
mono listening rows at `0.75`, `1.25`, and `1.5`. It must pass objective
non-regression and improve `L001` crest by at least `3 dB` before listening.

## Stop Rule

Only repeatable objective and concealed-listening value can reopen automatic
selection. Failure or neutral listening retires the time-adaptive successor.

## Next Task

Run Batch 29.6BA oracle adaptive synthesis and targeted mono gate.
