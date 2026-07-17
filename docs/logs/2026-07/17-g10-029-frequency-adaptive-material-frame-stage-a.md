# g10.029 Frequency-Adaptive Material Frame Stage A

Date: 2026-07-17
Batch: 29.7Y Stage A
Status: complete

## Scope

Prove the selected frequency-adaptive representation before any material phase
or time stretch. Use the frozen `4096/2048/1024` supports and `750 Hz`/`6 kHz`
crossovers. Do not tune after failure.

## Evidence

The report-only `f64` frame uses a `16384`-frame transform, `8192`-frame source
crop, whole-sample reflection, `32` coefficients per band, and a common
`512`-frame lattice. Long, middle, and short scales exclusively own `127`,
`448`, and `769` atoms. One global canonical dual synthesizes all scales.

Untouched coefficients reconstruct with `3.04e-16` peak error and `7.48e-17`
RMS error. Frame bounds are `0.9999999999999999` and
`1.0000000000000002`; conjugate closure is `6.97e-13`. Coverage, local
coefficient assignment, finite values, exact crop, silence, reflected head and
tail impulses, hard pan, swap, polarity, scaled duplicates, and deterministic
repeat all pass with zero failures. Evidence hash: `35b893204a56fcf3`.

## Decision

Stage A passes. Stage B may add the one frozen material-state phase policy to
this representation. Listening, production routing, dynamic ratio, realtime,
and Batch 29.8 remain closed until the objective candidate passes.

## Next Task

Run Batch 29.7Y Stage B. Implement the complete shared fuzzy material map,
transient shoulder/reset law, retained common-region rotation, and deterministic
channel-common noise perturbation. Run exactly one objective candidate at the
three frozen ratios without parameter rescue.
