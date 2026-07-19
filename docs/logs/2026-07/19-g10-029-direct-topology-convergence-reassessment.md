# g10.029 Direct Topology Convergence Reassessment

Date: 2026-07-19
Batch: 29.7BD
Contract: 082, Rules 31AE and 31AF
Status: complete

## Frozen Inputs

- AX corrected DSP `f680947a`, preregistration `255427a5`, and execution
  closeout `dad51432`
- AX stereo evidence `397128c177d3033e`
- BC mechanics at `4d668379` plus behavior-neutral split `552b3892`
- BC preregistration `8098a652` and execution closeout `eadbe9a6`
- BC synthetic evidence `ce696ab8cb37b17f`
- BC stereo evidence `b13c37cff1b58afa`
- unchanged objective thresholds and stereo-first stop state

No renderer or corpus command ran. Retained reports were read in place.

## Result

BC changes every candidate row hash but does not change the dominant result:
local failures remain `36/48` and maximum residual remains
`0.7611955347641768`. Two image rows gain one local window each; two other
image rows cross the calibrated mid/side limit in the wrong direction. No
local-failure classification changes.

All `24` tone rows remain local failures. Comparison-aligned tones have
interior IPD `1.153146..2.343541`, zero improved windows, and maximum local
residual `0.761196`. Comparison-unaligned tones have interior IPD
`0.00000298..0.00037522` and `39` improved windows.

The labels are inverted relative to Signal's direct scale. At `8 kHz`, the
aligned `246.09375 Hz` tone is `0.3125` direct long-scale bin from its nearest
bin; the unaligned `248.984375 Hz` tone is only `0.08125` bin away. Both are
long-scale only.

Code audit finds raw fuzzy material ratios commit directly to atom-local phase
states. Source-studied R3 inserts H/P/R labeling plus modal frequency
completion before state ranges. Select that missing ownership seam for one
bounded no-audio contract. Close further peak-ownership correction work.

## Next Task

Run Batch 29.7BE. Freeze one clean-room material label, modal completion,
frequency-range, tie, and coefficient-only proof. Keep implementation and all
audio closed.
