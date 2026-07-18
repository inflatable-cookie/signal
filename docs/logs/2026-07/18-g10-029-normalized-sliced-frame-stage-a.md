# g10.029 Normalized Sliced Frame Stage A

Date: 2026-07-18
Batch: 29.7AM
Status: passes

## Result

The release-only Rule 31T proof prepares and renders the normalized sliced
frame at `8`, `44.1`, and `48 kHz`. The exact `(N,A,H)` rows are
`(2560,1280,80)`, `(14112,7056,441)`, and `(15360,7680,480)`. Signed and
nonnegative atom counts are `380/191`, `1182/592`, and `1260/631`; tap counts
are `4740`, `27042`, and `29460`. All rows retain `K=32`, exhaustive scale
ownership, a positive frame operator, and the canonical inner dual.

Peak combined identity error is `4.440892098500626e-16`. Outer sine square-
partition error is `6.661338147750939e-16`; conjugacy error is zero. Short,
nonaligned, boundary-impulse, and multislice cases pass at every rate. Crop,
two-layer coverage, silence, hard pan, swap, polarity, scaled duplicate,
reflection, finite, and repeat checks pass. Evidence hash:
`0407f765c7d84375`.

## Bounds

The three coefficient terms are `194560`, `605184`, and `645120 Complex64`
slots. The `48 kHz` per-slice, per-channel structural row records two full
transforms, `2520` band transforms, `58920` tap visits, `80640` coefficient
visits, `61440` sample/window visits, and `7681` conjugate visits. Every memory
and work row matches Rule 31T.

Three duration rows per rate exercise `3`, `6`, and `14` required slices and
`64`, `112`, and `240` global token updates. The token crosses `4`, `7`, and
`15` outer boundaries. Active high-water is two. Reset, duplicate-update,
missed-update, and capacity failures are zero. All nine unsupported/capacity
cases return before processing; overflow failures are zero. No duration-sized
working store exists.

## Boundary

This is a representation and mechanics result, not a sound-quality result.
No guided material policy, stretched audio, objective row, listening artifact,
or holdout access exists. Rule 31U freezes the passing geometry, hash,
work/memory ceilings, and overflow behavior before synchronized channel-state
mechanics cross the slice boundary.

## Validation

- focused release proof: `2` passed
- legacy sliced-frame release regression: `1` passed
- guided linked-phase release regression: `3` passed
- `signal-dsp-stretch` debug suite: `269` tests passed across library, binary,
  integration, and documentation targets
- package format and missing-docs library checks passed
- `effigy doctor` remains at the pre-existing god-file and attention-marker
  baseline; the new modules add no finding

## Next Task

Run Batch 29.7AN under Rule 31U. Prove reset, attack, ordinary, unlocked, and
compatible locked state branches plus duplicate, mono-parity, silent-peer, and
swap mechanics before, at, and after slice creation and retirement. Keep
material policy, stretched quality audio, objective evidence, listening,
holdout, Batch 29.7AO, Batch 29.8, and product work closed.
