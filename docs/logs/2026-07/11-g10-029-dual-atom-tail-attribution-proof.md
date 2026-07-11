# g10.029 Dual-Atom Tail Attribution Proof

Date: 2026-07-11
Status: passed; planning checkpoint reached

## Result

Batch 29.6O attributes the rejected common-grid tail to boundary response
construction, not the canonical-dual solver.

## Evidence At Radius 16000

| Channel | Raw real tail | Tightened real tail | Dual real tail |
| ---: | ---: | ---: | ---: |
| 0 | `1.622121e-13` | `6.270779e-7` | `6.270779e-7` |
| 15 | `0` | `0` | `0` |
| 16 | `0` | `0` | `0` |
| 768 | `0` | `0` | `0` |
| 1535 | `1.180453e-7` | `1.699919e-7` | `2.030199e-7` |

- channel `0` tightening amplification: `3865790.426x`
- channel `0` dualization amplification: `1.000000000248x`
- channel `1535` tightening amplification: `1.440056526x`
- channel `1535` dualization amplification: `1.194291604x`
- maximum dual residual: `9.524707e-11`
- non-finite atom values: `0`
- complete matrix: `30/30` atoms
- repeated evidence and hashes: exact
- report hash: `5369c9fcaa33334c`

Raw channel `0` gains strong tail cancellation when conjugate-mirrored for real
output. Tightening breaks that cancellation. Channel `1535` has a separate raw
Nyquist-edge tail, so removing tightening alone cannot close the guard.

## Boundary

No filters were changed. No coefficient assembly, audio synthesis, corpus,
stereo, dynamic ratio, or product route opened.

## Next Task

Freeze one joint DC/Nyquist boundary-completion design while preserving the
passing interior bank.
