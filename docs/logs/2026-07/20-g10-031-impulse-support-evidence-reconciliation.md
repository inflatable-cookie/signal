# g10.031 Impulse Support Evidence Reconciliation

Date: 2026-07-20
Batch: 31.35
Status: complete; fresh candidate authority frozen

## Decision

Batch 31.34 is an executable-evidence construction failure, not renderer
support evidence. Its frozen test scanned complete impulse output for dropout.
The audited authority reserves complete impulse output for first-difference
crest and limits dropout to mapped authored support.

The exact source supports are frozen once in `SYNTHETIC_SUPPORTS`. For source
support `[a,b)`, source length `L`, and target length `T`, dropout maps to
`[floor(a*T/L),ceil(b*T/L))` with checked `u128`. It examines only complete
`H`-sample windows wholly inside that range.

The isolated impulse owns `[48000,48001)`. At `4x`, `8x`, and `16x`, its
mapped hull contains `4`, `8`, and `16` frames. All are shorter than
`H=16384`, so no dropout window exists. `Y03` remains the impulse spread and
placement owner. `Y04` remains the replica owner. The complete-output
discontinuity gate remains terminal.

Batch 31.25 passed the intended `Y08` under the otherwise matching mono
topology. Checkpoint `f76d5bb7` remains rejected and deleted; no source, tests,
checkpoint, or receipt were recovered or reinterpreted.

Fresh authority is
`SupportAuditedListeningLedSourceRelativeRenewalSpectral`. It retains the
renderer formulas, sources, thresholds, `ADMISSION_SEED`, structural and
synthetic owners, concealed mono pack, and linked-stereo admission. Only
support ownership and executable range construction are made unambiguous.

No DSP, candidate harness, test, fixture, API, cache, route, product surface,
Loophole, or Chorus change entered `main`.

## Validation

- `git diff --check`: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass

## Next Task

Run Batch 31.36 only. Create `signal-candidate-31-36` on
`candidate/g10-031-support-audited-listening-led-renewal`. Implement the frozen
brief once from fresh source, complete compile and construction `1/1`, freeze
one checkpoint, then run structural and synthetic admission in order. Do not
recover rejected source, change gates, admit product surfaces, or push.
