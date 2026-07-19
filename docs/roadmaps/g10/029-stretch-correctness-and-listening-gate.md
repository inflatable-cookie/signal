# 029 - Stretch Correctness And Listening Gate

Status: superseded by `g10.030`
Owner: dsp
Created: 2026-07-09
Closed: 2026-07-19
Governing contracts: historical `082`; current `046` and `084`

## Outcome

This roadmap corrected the OfflineHighQuality boundary path, established
full-render integrity measurement, built external comparison and blind
listening workflows, promoted bounded compression and expansion short-window
selectors, studied professional reference implementations, and rejected a
large sequence of successor candidates.

It then drifted into narrow proof churn. The 2026-07-19 consolidation closed
that sequence, removed its rejected code, and cancelled Batch 29.7BE.

The complete pre-consolidation ledger remains in git history at `1d1b02f1`.

## Retained Product Results

- centered OfflineHighQuality boundary coverage and exact target cropping
- full-render length, endpoint-energy, added-silence, and peak-growth metrics
- event-level transient timing and crest diagnostics
- tonal-texture and formant-boundary diagnostics
- Signal-versus-external rendered-output comparison
- five-family, three-ratio level-matched blind listening pack
- compression and expansion short-window production selectors
- byte-exact baseline, linked-stereo, pitch, dynamic-ratio, cache, artifact,
  and RealtimePreview regression coverage

## Operator Findings

The frozen Signal baseline is competitive with Rubber Band on the retained
material, with no universal winner. Its clearest remaining weaknesses are:

- slightly grainier long expansion
- subtle atonal ringing on some material
- marginally softer or less stable transients
- occasional transient pop spikes visible in the waveform
- uncertain small transient-placement differences

Short stabs were insufficient to judge musical quality reliably. Long-form,
level-matched comparisons are the promotion authority. Independent stereo
listening remains outstanding because the primary operator cannot perform it.

## Rejected Directions

The following did not earn production routing and were removed:

- local phase-lock stability, tracked-peak, magnitude-slew, and compression
  anchor variants
- tail anchoring, zeroing, fading, and tail-local selection
- long-window blends and envelope matching
- structural branch hybrid and adaptive transient timeline
- fixed-map peak transient treatment
- iterative H/R/P separation and additive rendering
- whole-band phase-gradient integration
- common-grid and frequency-adaptive transform families
- the later direct multiscale, material-state, and linked-stereo proof sequence

Some mechanisms passed isolated structural gates. None passed the complete
sound-quality and stereo admission sequence. They remain research evidence,
not implementation candidates.

## Consolidation

- `43e9a96a`: removed `50,397` lines of frequency-adaptive research
- `1d1b02f1`: removed `16,103` lines of remaining rejected renderer and report
  machinery
- production bit-exact baseline remained unchanged
- retained `signal-dsp-stretch` tests passed

## Authority

Do not execute old Batch 29.x instructions from git history. Contract `084`
and roadmap `g10.030` require one complete successor in an isolated branch or
worktree. Failed candidates do not accumulate in `main`.

## Next Task

Use `g10.030` Batch 30.2 to freeze one end-to-end successor brief before new
DSP implementation.
