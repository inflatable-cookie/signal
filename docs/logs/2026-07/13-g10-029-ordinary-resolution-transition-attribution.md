# g10.029 Ordinary Resolution And Transition Attribution

Date: 2026-07-13
Batch: 29.6CA
Rule: 30V

## Scope

Kept the nine Rule 30U development rows, ratios, time map, ordinary phase
policy, diagonal-dual synthesis, and measurements frozen. Rendered current
Signal, fixed `512`, `1024`, `2048`, and `4096` ordinary controls, and adaptive
ordinary synthesis. No holdout read, listening export, tuning, detector or
schedule change, stereo, dynamic ratio, cache, or routing work.

## Result

The defect has three owners.

- Endpoint integrity is resolution-dependent. Hard failures by mode are
  `[0,9,9,4,0,7]` for current, fixed `512`, `1024`, `2048`, `4096`, and
  adaptive. Adaptive makes `214` resolution changes.
- Spectral/formant damage is shared. Every fixed length and adaptive ordinary
  synthesis regresses both static-spectrum and formant residual in all nine
  rows against current Signal.
- Adaptive transitions add timing damage. Mean timing deltas from current are
  `+129.138889`, `+179.444444`, `+34.916667`, `+82.027778`, and
  `+196.166667` frames. Adaptive timing is worse than fixed `512`, `1024`,
  `2048`, and `4096` in `6`, `5`, `7`, and `6` rows.

The result rejects a single fixed-window explanation. Fixed `4096` provides a
clean-integrity control for the next shared-mechanism factor study, but it is
not a production selection: it still regresses static-spectrum and formant
residual in `9/9` rows.

## Frozen Evidence

- rows: `9`
- modes: `6`
- renders: `54`
- repeatability pass: `108` renders
- changed from current: `[9,9,9,9,9]`
- fixed changed from adaptive: `[9,9,9,9]`
- manifest hash: `c4cde9a638c1e36e`
- render hash: `9a3ff69ddc1dc765`
- measurement hash: `3e4f4a8489a8217d`
- aggregate hash: `c00d6c130888505a`
- TSV: `target/stretch-successor-ca-resolution-attribution.tsv`
- TSV SHA-256:
  `b5e16237c11d4733e874ac09d1ca41007690518c915a48fed1c00cd4c07b5ace`
- holdout reads: `0`
- listening exports: `0`

## Next Task

Execute Batch 29.6CB under Rule 30W. On fixed `4096`, factor ordinary phase
transport, event-warped versus global-linear output placement, and
diagonal-dual overlap synthesis. Keep holdout, listening, tuning, window
selection, detector/schedule policy, stereo, dynamic ratio, cache, and routing
closed.
