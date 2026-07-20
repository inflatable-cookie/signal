# g10.031 Audited Variance-Renewal Brief

Date: 2026-07-20
Status: Batch 31.24 complete; Batch 31.25 ready
Roadmap: `g10.031`
Contract: `085`

## Decision

Retain the compensated-renewal topology. Batch 31.23 established no valid
synthetic or listening result, so changing or abandoning its source-backed DSP
would be unsupported.

Freeze fresh identity `AuditedVarianceCompensatedRenewalSpectral`. Its source
must be implemented cleanly; deleted candidate code and tests may not be
recovered.

## Evidence Repair

The new complete brief freezes:

- `22` compile-linked gate owners: `13` structural and `9` synthetic
- one construction-manifest test before the immutable checkpoint
- exact source fades, mapped supports, metric algorithms, row counts, and gate
  commands
- shortest 95%-energy impulse widths of `79,469`, `155,953`, and `309,239`
- sample-centred impulse-event references
- every autocorrelation lag from `960` through `48,000`
- explicit one-region/no-secondary replica references for impulse and impulse
  train
- PaulX-relative first-difference crest as the discontinuity control
- actual allocation accounting, including FFT plans, plus zero allocation or
  reallocation after frame processing starts

Retained PaulX raw renders supplied the corrected and newly explicit reference
values. No candidate render or listening audio was created.

## Boundary

No DSP, public API, report, binary, fixture, dependency, cache, route,
Loophole, or Chorus surface changed. Batch 31.25 remains one isolated private
candidate. Mono listening may open only after a valid structural and synthetic
receipt. Independent stereo listening remains mandatory later.

## Validation

- documentation-only batch
- known `effigy doctor` god-file and attention-marker findings unchanged

## Next Task

Run Batch 31.25 only. Implement the frozen audited brief once in
`signal-candidate-31-25`, pass compile and construction-manifest validation,
then create the immutable checkpoint before structural admission.
