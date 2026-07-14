# g10.029 Long-Form Listening Decision

Status: complete
Date: 2026-07-14
Batch: 29.6CK

## Decoded Result

| Row | Weighted predictor | Current Signal | Rubber Band R3 |
| --- | --- | --- | --- |
| M001 drums 1.5x | altered early bass tone | slightly grainy | cleanest |
| M002 bass 1.5x | cleanest | grainier | good, less crisp |
| M003 vocals 2.0x | cleaner, MP3-like edge | quite grainy | best |
| M004 pads 2.0x | severe MP3-like damage | grainy | good |
| M005 mix 1.5x | slightly best | grainy | similar |
| M006 mix 2.0x | acceptable | very grainy and blurry | best |

Weighted prediction beats current Signal on M002, M003, M005, and M006. This
is the first coherent successor improvement in the long-form program. It does
not establish promotion: M001 changes tone, M004 is badly damaged, and Rubber
Band wins four rows.

## Mechanism Finding

The Signal proof implemented a simplified sketch, not the studied topology:

- `2048/128` window/hop rather than sample-rate-scaled 120/30 ms geometry
- same-frame neighbour phase offsets rather than local-time-factor-scaled input
  frequency twists
- horizontal prediction included directly in an ad-hoc magnitude sum rather
  than a separate vertical re-prediction
- no prediction-energy normalization or weak-evidence input fallback
- independent per-bin sums rather than the specimen's dependency-aware update

This is a different architecture. The observed bass mutation and pad damage are
therefore attributed to an unresolved predictor-fidelity gap, not to a window,
distance, or weight value awaiting a sweep.

## Decision

- validate weighted prediction as the successor foundation
- reject the current proof for promotion
- freeze one faithful Signal-owned topology before more synthesis code
- keep random diffusion, frequency partitioning, parameter search, real-source
  rendering, holdout, stereo, dynamic ratio, cache, and routing closed

## Next Task

Batch 29.6CL defines the complete predictor mechanism and direct synthetic
bass-tone, chord/pad, transient, silence, boundary, determinism, finiteness, and
exact-length gates.
