# Linked-Stereo Contract

Date: 2026-07-16
Roadmap: `g10.029`, Batch 29.7A
Scope: report-only coherent predictor stereo ownership and gates

## Decision

Extend the competitive coherent mono predictor through one shared frame loop.
Share schedule, geometry, traversal, neighbour availability, and aggregate
corrected/fallback mode. Keep analysis, recurrence, magnitudes, synthesis, and
normalization per-channel.

## Ownership

Shared:

- fixed ratio, sample rate, source/output centres, target length, and crop
- periodic Kaiser window and modified half-bin transform
- ascending bin traversal and short/long neighbour availability
- aggregate corrected-versus-fallback mode from summed channel energies

Per-channel:

- current and auxiliary spectra
- previous input energy and corrected output
- horizontal and vertical complex predictions
- target magnitude, numerical fallback phase, accumulation, and normalization

Mid/side resynthesis, dominant-channel phase replacement, sample crossfeed,
independent frame schedules, dynamic ratio, product routing, and realtime use
are excluded.

## Proof Runway

1. Batch 29.7B: mechanics, mono parity, transformations, structure, shared-mode
   exercise, crossfeed, unilateral completion, and repeat.
2. Batch 29.7C: constant IPD, broadband delay, mid/side ratio, correlation,
   one-sided transients, and frozen quality hashes.
3. Batch 29.8: stereo export and independent review only if both proofs pass.

## Stop Conditions

- any frozen mono hash changes
- duplicated-mono or hard-pan parity fails
- non-finite, uncovered, boundary, length, crossfeed, or repeat failure
- any non-silent unilateral completion in the mechanics controls
- mechanics require a new threshold, dominant phase owner, or parameter sweep

## Next Task

Implement Batch 29.7B in a narrow faithful-predictor stereo module or shared
frame core. Add focused tests for the complete mechanics gate. Do not run the
quality controls until mechanics pass.
