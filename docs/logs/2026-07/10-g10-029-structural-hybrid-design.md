# g10.029 Structural Hybrid Design

Date: 2026-07-10
Status: design frozen; kernel seam complete; mono candidate ready

## Code Map

- `OfflineHighQualityStretcher` in `src/lib.rs` owns public fixed-ratio,
  pitch-composed, stereo, path-selection, and dynamic-ratio orchestration.
- `DraftPhaseVocoder` in `src/phase_vocoder.rs` currently owns analysis,
  transient evidence, peak tracking, phase propagation, spectrum construction,
  overlap-add synthesis, normalization, boundary padding, and output cropping.
- fixed-ratio stereo converts left/right to mid/side, then runs two independent
  mono engines. It shares parameters, but not transient, peak, or phase
  decisions.
- dynamic ratios render independent source segments, concatenate them, then
  smooth the joins over `256` frames. Algorithm state does not cross a ratio
  boundary.
- `detect_stretch_transients_with_policy` in `src/benchmark.rs` is a
  whole-render measurement primitive. Its global normalization and event-list
  output are not synthesis policy.
- centred padding and exact cropping live in `run_phase_vocoder`; every
  successor branch must retain that boundary contract.

The present core is adequate for bounded experiments but too entangled for a
multiresolution engine. Stereo and dynamic-ratio claims are also narrower than
their names suggest.

## First Hybrid Shape

The first candidate is fixed-ratio and report-only. It runs bounded branches
continuously so phase state does not restart when ownership changes.

| Owner | Window / hop | Propagation | Scope |
| --- | --- | --- | --- |
| Transient | `1024 / 256` | independent-bin | compression and expansion transient guards |
| Mixed | `2048 / 512` | current identity-lock/reset policy | uncertain regions and all boundary guards |
| Tonal | `4096 / 1024` | identity-lock/reset policy | stable expansion regions only |

This is local ownership, not a whole-render selector. Compression does not use
the tonal branch in the first candidate because the long-window evidence was
qualified only for expansion.

## Classifier

The candidate adds an internal `HybridFrameClassifier`; it does not reuse the
benchmark detector as synthesis policy.

- transient evidence uses short-window positive spectral-flux ratio `>= 0.30`
  and energy ratio `>= 1.20`, matching the current expansion-reset evidence
- a transient guard starts one short hop before the detected frame and remains
  active through three short hops after it
- tonal evidence requires spectral stability `>= 0.70` for four consecutive
  short hops, outside every transient guard
- `Transient` owns guarded frames, `Tonal` owns qualified expansion frames,
  and `Mixed` owns everything else
- a transient immediately exits tonal ownership; tonal ownership must qualify
  again after the guard

These constants are frozen for the first candidate. A failed gate rejects this
candidate shape; it does not open a threshold sweep.

For stereo, energy and flux are measured per channel and the maximum normalized
evidence drives one shared state. Tonal stability and peak support use summed
channel power. One-sided and anti-phase attacks therefore cannot disappear in
a mid downmix.

## Transition Law

- every branch maps analysis-window centres to the same exact output timeline
- the first and last half-default-window spans remain `Mixed`
- only two owners may overlap during a transition
- ownership changes use a `256`-sample raised-cosine crossfade whose linear
  amplitude weights sum to one
- the crossfade centre moves by at most one short hop to the lowest current-path
  power outside a transient guard
- a transition is eligible only when outgoing/incoming zero-lag correlation is
  at least `0.50` and correlation-aware energy normalization needs no more than
  `1 dB` correction; otherwise `Mixed` owns the region
- transition normalization uses one gain for both stereo channels
- branch state continues while its output weight is zero; ownership changes do
  not reset analysis or synthesis phase
- candidate reports expose class spans, transition positions, selected
  low-energy offsets, branch correlation, normalization gain, branch weights,
  rejected transitions, and changed-output energy

This is not the rejected fixed half-blend. Mixing is confined to declared
ownership transitions and must be measured for cancellation, crest movement,
and clicks.

## Linked Stereo

The linked candidate must use one multichannel core. Converting to mid/side and
calling a mono engine twice is insufficient.

- one class schedule, lane schedule, transition schedule, and reset schedule is
  shared by both channels
- summed left/right power defines shared spectral peaks and region bounds
- each channel retains its own instantaneous-frequency propagation
- at a shared peak, the dominant channel advances the reference phase; the
  other channel retains the wrapped analysis interchannel phase difference
- identity locking uses the shared region bounds while preserving each
  channel's relative analysis phase inside the region
- a shared transient reset restores each channel's own analysis phase on the
  same frame
- mono uses the same core with one channel; no stereo-only algorithm fork is
  allowed

The validation surface must add local interchannel phase-error evidence around
shared peaks and attacks. Whole-render correlation and side/mid ratio remain
required but are not sufficient.

## Formant, Pitch, Dynamic Ratio, And Boundary Policy

- no formant correction enters fixed-ratio no-pitch stretch; no measured
  failure supports one
- pitch composition stays on the existing path until fixed-ratio hybrid
  evidence passes; later formant work requires pitch-shift vocal evidence
- dynamic ratio stays on the existing segmented prototype. A hybrid successor
  must carry classifier, branch, phase, and synthesis cursors continuously
  through ratio changes instead of rendering and joining independent segments
- every branch retains centred padding, exact cropping, deterministic output
  length, and identity bypass
- the hybrid does not add a tail envelope. Exterior-tail behavior is measured
  as an algorithm outcome

## Validation Matrix

Kernel-seam validation:

- bit-exact current default output before and after internal extraction
- deterministic class and transition traces
- exact length, identity, centred-boundary, finite-output, and no-empty-span
  invariants for each branch

Mono candidate gate:

- improve the anchored `L001` crest by at least `3 dB`
- do not move the worst crest above the current `5.655 dB` maximum
- do not worsen corpus mean absolute event placement by more than `1` frame
- reduce mean excess spectral movement at both `1.25x` and `1.5x`
- do not increase source-relative static residual or unsupported-bin mass at
  either expansion ratio
- retain `60/60` integrity, transient, tonal, formant, boundary, and combined
  passes

Linked-stereo gate:

- mono/stereo-identical input parity within floating-point tolerance
- no worse whole-render image delta than the current linked path
- bounded shared-peak interchannel phase error with no one-sided transient miss
- one shared class and transition trace for both channels
- independent stereo listening remains mandatory before production routing

## Stop Conditions

- reject the candidate if the local crest improvement moves the worst event or
  creates a corpus timing regression outside the gate
- reject it if either expansion ratio fails to improve fast spectral movement
  without static-spectrum regression
- reject the transition law if a crossfade creates a new click, thump, crest,
  material cancellation, or normalization outside the `1 dB` bound
- reject linked stereo if shared decisions worsen image or interchannel phase
  evidence
- do not tune classifier constants after a failed combined gate; reassess the
  ownership mechanism
- do not change production, cache identity, product receipts, pitch routing,
  dynamic-ratio routing, or RealtimePreview support in these report-only batches

## Execution Runway

1. Batch 29.5 extracts a reusable branch kernel and adds classifier/transition
   traces while proving the current default remains bit-exact.
2. Batch 29.6 implements and gates the fixed-ratio mono hybrid candidate.
3. Batch 29.7 adds the shared-decision linked-stereo core only if mono passes.
4. Batch 29.8 runs concealed listening and decides whether stateful dynamic
   ratio design may open. A failed mono or stereo gate returns to structural
   reassessment instead.

## Next Task

The first Batch 29.6 candidate was rejected by the frozen mono gate. Re-enter
structural reassessment; do not tune classifier constants or open linked
stereo. See
`docs/logs/2026-07/10-g10-029-fixed-ratio-mono-hybrid-rejection.md`.
