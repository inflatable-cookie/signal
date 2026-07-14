# Coherent Representation Synthetic Gate

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CY
Scope: report-only faithful-predictor synthetic proof

## Decision

Pass the coherent periodic-Kaiser/modified-half-bin representation through the
complete synthetic gate. Open exact-input real-source objective confirmation.

The implementation injects the representation into the existing faithful-
predictor review. Scheduling, predictor equations, distances, energy law,
fallback, boundary policy, synthesis ownership, and production selection do
not change.

## Evidence

- geometry: `960/240` support/interval, `1024` transform, `512` bands
- structural failures: `[0, 0, 0, 0, 0]`
- maximum bass error: `0.000718348 Hz`
- octave failures: `0`
- maximum chord peak error: `0.007313938 Hz`
- chord input/output out-of-band energy: `-80.429254/-40.664119 dB`
- maximum transient placement error: `1 frame`
- transient replica failures: `0`
- silence peak: `0`
- source-relative tone/chord failures: `[0, 0]`
- complete-proof hash: `0905a7fd4180bff4`
- repeat: exact

Mechanism counts:

- horizontal: `360448`
- short lower/upper: `359744/359744`
- long lower/upper: `357632/357632`
- corrected/fallback: `260556/99892`

The frozen source-parity hashes remain unchanged from Batch 29.6CX.

## Consequence

The complete source-derived analysis basis is coherent beyond isolated-tone
fidelity. It preserves the existing predictor's structural, tonal, transient,
weak-evidence, and deterministic behavior. Batch 29.6CZ may now test identical
musical inputs against pinned Signalsmith at the long-form sample rate.

## Closed Lanes

- predictor-law changes, third mechanisms, and parameter sweeps
- listening, stereo, dynamic ratio, product routing, and promotion

## Next Task

Run Batch 29.6CZ exact-input real-source confirmation.
