# Rubber Band Behavioural Probe Contract

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BD`
Status: complete; measurement ready

## Installed Specimen

- executable: `/opt/homebrew/bin/rubberband`
- version: `4.0.0`
- R2 and R3 CLI modes: available
- R2 transient, lamination, detector, and window controls: available
- R3 short-window restriction: available
- public headers: `/opt/homebrew/include/rubberband`
- dynamic and static libraries: `/opt/homebrew/lib/librubberband*`
- `pkg-config` metadata: unavailable

Batch 29.6BE must record these as a capability receipt. It must not assume the
same paths on another machine.

## Mono Matrix

Every mono control runs at ratios `[1.0, 0.75, 1.25, 1.5]` in all five modes.

| Mode | Arguments | Contrast |
| --- | --- | --- |
| `r2-default` | `--fast --crisp 5` | R2 reference |
| `r2-no-reset` | `--fast --no-transients` | transient resynchronization enabled versus disabled |
| `r2-no-lamination` | `--fast --no-lamination` | lamination enabled versus disabled |
| `r3-standard` | `--fine` | full R3 multi-resolution mode |
| `r3-short` | `--fine --window-short` | R3 restricted single-window mode |

Controls:

1. `bass-tone`: `55 Hz`, bin-stable duration
2. `mid-tone`: `440 Hz`, bin-stable duration
3. `two-tone`: separated low and high partials
4. `linear-chirp`: continuous low-to-high sweep
5. `hard-impulse`: one declared broadband event
6. `soft-onset`: deterministic raised-cosine attack
7. `dense-impulses`: two declared events `256` frames apart
8. `boundary-impulses`: declared first and final active events
9. `tonal-impulse`: stable low tone plus central impulse
10. `noise`: deterministic stationary broadband noise
11. `complex-mix`: bass, moving partials, soft onset, and impulses
12. `silence`: numerical and duration control

This produces `12 * 4 * 5 = 240` mono render rows.

## Stereo Matrix

Run ratios `[0.75, 1.5]` in `r2-default`, `r3-standard`, and `r3-short`.

1. linked identical hard impulse
2. unequal-channel tonal-plus-impulse mixture
3. centre tone plus side events
4. anti-phase tonal control

This produces `4 * 2 * 3 = 24` stereo rows. Total expected rows: `264`.

## Evidence Separation

Three record families remain distinct:

1. capability/render receipt: executable, version, requested mode, arguments,
   exit status, length, clipping, finiteness, hashes
2. direct public-API state: output increments, reset curve, exact-time points,
   engine confirmation, support status and reason
3. waveform inference: event displacement and local slopes, crest, replicas,
   endpoints, silence, vertical coherence, stereo phase, tonal texture, hashes

No inferred field may populate direct-state evidence. No direct-state field is
required when the adapter is honestly unsupported.

## Contrast Limits

The mode differences are system-level contrasts. R2 no-reset may change both
phase and local time allocation. R3 short changes the complete restricted
window policy, not only one FFT length. Attribution requires repeatable
direction across relevant control families and ratios.

## Determinism And Stop Conditions

- generate every source deterministically at `48 kHz`
- run every render and public-state query twice
- require identical manifest, command, state, sample, and measurement hashes
- require all five CLI modes and exact `264`-row coverage
- stop before mechanism attribution on a missing mode, failed render,
  non-finite sample, or repeat mismatch
- public-state adapter absence does not block waveform measurement when its
  unsupported receipt is complete
- do not add licensed corpus rows until the synthetic report passes

## Next Task

Implement Batch 29.6BE in the existing stretch comparator harness. Do not build
a Signal synthesis candidate.
