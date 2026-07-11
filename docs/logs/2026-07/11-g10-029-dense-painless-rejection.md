# g10.029 Dense Painless Common-Lattice Rejection

Date: 2026-07-11
Status: rejected; operator review required

## Geometry

The release-only `65536`-frame proof rebuilds the Batch 29.6I bank unchanged:

- bands: `832`
- common coefficient count: `16384`
- common hop: `4`
- unequal-lattice coefficients: `187386`
- dense coefficients: `13631488`
- growth: `72.7454985965x`
- redundancy: `208`

Filter, frame-operator, and pointwise-dual hashes match their same-length
unequal-lattice baselines. There are no uncovered bins, painless-support
violations, or non-finite values.

## Passing Evidence

- frame bounds: `0.9999999176` / `1.0000000832`
- condition: `1.0000001657`
- reconstruction peak: `5.5511151231e-16`
- reconstruction RMS: `1.3364241355e-16`

## Rejection

Real-spectrum closure is `1.7881393433e-7`, above the frozen `1e-12` cap.
Neither analysis nor dual atoms reach `1e-12` excluded energy within radius
`16384`. The limiting band is `830`; both cap ratios are `0.4999847412`.

Dense regridding fixes row alignment but not time localization. Cost is also
material, but no post-measurement cost threshold is used in the rejection.
Evidence hash `e0cbc3c75529c899` repeats exactly.

## Next Task

Stop for the Batch 29.6AH operator direction checkpoint. No transform, phase,
or synthesis implementation is authorized.
