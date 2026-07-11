# g10.029 Boundary Preconditioner Rejection

Date: 2026-07-11
Status: rejected at reconstruction conditioning

## Result

Batch 29.6R implemented the single frozen common scalar normalizer. It uses
exact inverse-square-root pointwise frame energy in the interior and quintic
endpoint blends over the fixed `16h` spans. The raw boundary bank is unchanged.

Release-profile reconstruction evidence:

- frame minimum: `0.4649443040681184`
- frame maximum: `1.40346349491382`
- condition ratio: `3.0185626162831762` (required at most `1.25`)
- canonical-dual residual: `7.899949442472469e-11`
- reconstruction peak error: `7.275957614183426e-12`
- reconstruction RMS error: `1.9925657301942398e-13`
- reconstruction head error: `7.048637932008384e-13`
- reconstruction tail error: `0`
- non-finite values: `0`
- raw-bank hash: `c1014f5fc308c290`
- multiplier hash: `fd32b38fb8e92972`

Evidence and hashes repeat exactly in the release proof. The raw-bank hash also
matches the unmodified Batch 29.6P bank in the same build.

## Stop Decision

The reconstruction condition gate fails. The representative and all-channel
guards did not run. Phase reproof, coefficient assembly, audio synthesis,
corpus, stereo, dynamic ratio, cache, and product routing remain closed.

Endpoint smoothness is insufficient because the complete decimated transform
is governed by alias-block eigenstructure, not pointwise energy alone. Do not
retune the blend or add channel gains. The next evidence must identify the
limiting residue blocks, boundary bins, and channel/cross-term ownership before
another preconditioner or boundary geometry is authorized.

## Next Task

Freeze Batch 29.6S alias-block conditioning attribution. Do not implement it in
this batch.
