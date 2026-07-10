# g10.029 Common-Grid Wavelet Reconstruction Proof

Date: 2026-07-10
Status: passed

## Change

Added a report-only analytic Cauchy wavelet bank with `1536` channels, `16`
lowpass completion channels, uniform `384`-frame decimation, digital `(0,1)`
channel delays, deterministic frequency-response tightening, and a complete
alias-block canonical-dual solve.

No phase modification, time stretch, corpus render, or product route changed.

## Evidence

The `4096`-frame mixed control pads to `4224` frames and produces a `1536 x 11`
coefficient matrix:

- redundancy: `8`
- delay hash: `a6e29251d820b406`
- estimated frame bounds: `0.984806890` / `1.010234560`
- condition ratio: `1.025819956`
- maximum canonical-dual residual: `6.225219e-11`
- peak reconstruction error: `2.910383e-11`
- RMS reconstruction error: `5.520117e-13`
- coefficient hash: `4324c481e2350802`

Low, middle, high, `19.5 kHz`, `23.5 kHz`, impulse, deterministic noise, mixed,
silence, empty-input, finite-value, endpoint, and repeat controls pass.

## Boundary

The coefficient rows now share one time grid. Phase transport still needs an
explicit contract for channel-delay compensation, unequal centre bandwidths,
cross-channel integration, synthesis positions, and real-output symmetry.

## Next Task

Research and contract the common-grid phase mechanism and synthetic stop gate.
