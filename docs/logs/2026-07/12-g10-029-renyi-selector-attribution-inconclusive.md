# g10.029 Rényi Selector-Failure Attribution

Date: 2026-07-12
Status: inconclusive

## Structural Evidence

- Batch 29.6AK gate failures: `[0,1,0,0,2,0,0]`
- Batch 29.6AK evidence hash: `5568f0a38f679a40`
- time/frequency count closure: exact
- maximum time-sum closure error: `5.4116953673e-16`
- maximum frequency-sum closure error: `9.8277925391e-14`
- non-finite, empty-removal, and baseline-drift failures: `0`

## Attribution

- diagnostic anchors: `[15,5,32]` isolated, mixed event, mixed negative
- event-facing time removal: `8/15` isolated anchors restored; `5/32` mixed
  negative controls changed
- folded-frequency event restoration: `[5,0,0,0,0,0,0,0]`
- folded-frequency negative changes: `[1,0,0,0,0,0,0,0]`
- linear-chirp time-removal changes: `[0,0,0,0,0,0,0,0]`
- linear-chirp frequency-removal changes: `[39,0,0,0,0,0,0,0]`
- passing geometry/frequency candidates: `[0,0]`
- attribution hash: `e0b4421038492480`

Both suspected mechanisms are measurable. Neither satisfies the frozen clean
ownership rule. No selector boundary opens.

## Next Task

Freeze Batch 29.6AN attribution reassessment. Do not change the selector or
implement phase or stretched synthesis.
