# 082 Offline Time-Stretch Synthesis Policy Contract

Status: active; fixed-map peak transient mechanism frozen
Owner: dsp
Updated: 2026-07-10
Related contracts: `046`, `048`, `049`
Related architecture: `docs/architecture/offline-time-stretch-synthesis.md`

## Purpose

Freeze the synthesis-policy boundary after the first multi-output structural
hybrid failed its mono gate. This contract governs algorithm proof work. It
does not promote a product path.

## Rules

### Rule 1: one synthesis timeline

The successor must own one monotonic source-to-output time map and one
reconstruction timeline. Different analysis resolutions must not produce
independent output waveforms for later branch crossfade or delay alignment.

### Rule 2: transient preservation keeps the global time map

The next proof uses the current `2048/512` synthesis grid and constant global
synthesis hop. It must not move synthesis-frame positions, create local
unity-ratio islands, or distribute compensation through later frames.

The frozen `g10.029` onset classifier supplies event guards only. Inside each
guard, a time-ramped companion analysis estimates per-bin group delay. Each
spectral peak owns the bins between its nearest surrounding magnitude minima.
The energy-weighted group delay of that region estimates the attack position
inside the analysis window.

### Rule 3: phase reinitialization is peak-selective

An attack peak may reinitialize only when its peak-local energy position
reaches the window-centre threshold derived for the analysis window. Peak bins
collected for one guarded event reinitialize together in one frame. Their
synthesis phases copy the current analysis phases. Other bins retain
instantaneous-frequency propagation and current identity locking.

The candidate must not apply a whole-frame reset, waveform crossfade, transient
amplitude boost, corpus-tuned threshold sweep, or different synthesis timeline.
Peak-local magnitude-minimum regions, event collection, and reset decisions
must be explicit report data.

### Rule 4: adaptive resolution is a later reconstruction stage

Short attack resolution and long stable-component resolution must share one
nonstationary frame schedule and compatible reconstruction weights. The
current independent `1024`, `2048`, and `4096` rendered branches remain
diagnostic controls only.

### Rule 5: exactness and evidence remain mandatory

Every proof retains identity bypass, centred boundary coverage, deterministic
output, finite samples, exact target length, and explicit mapping evidence.
The transient proof must report guarded events, candidate peaks, collected
peak regions, reinitialized bins and frames, group-delay threshold crossings,
unmatched guards, and current-versus-candidate quality.

### Rule 6: promotion stays closed

Production routing, cache identity, product receipts, pitch composition,
dynamic-ratio routing, RealtimePreview, and linked stereo remain unchanged.
Adaptive-resolution work opens only after the fixed-map peak transient proof
passes its declared gate.

## Current-Grid Transient Proof Gate

- improve anchored `L001` crest by at least `3 dB`
- keep the candidate worst crest at or below `5.655483 dB`
- do not worsen corpus mean absolute event placement by more than `1` frame
- retain `60/60` integrity, transient, formant, boundary, and combined passes
- do not regress source-relative residual or unsupported-bin mass
- no non-finite output, non-monotonic synthesis position, uncovered output
  sample, or unreported dense-transient fallback

This gate proves only the peak-selective transient mechanism. It does not waive
the fast spectral-movement gate required of the later combined mono candidate.

## 2026-07-10 Proof Outcome

The adaptive transient timeline failed: `L001` improved only `0.536217 dB`,
mean event placement worsened by `4.942263` frames, and the combined gate passed
`9/60`. Exact anchors and overlap-add coverage passed, but sparse onset anchors
required local hops up to `1664` frames and moved unprotected events. Do not
tune classifier or compensation constants and do not open adaptive resolution.

## 2026-07-10 Reassessment Decision

Use peak-local group-delay phase reinitialization under the unchanged global
time map for the next proof. This mechanism targets invalid transient phase
prediction and broad phase ownership inside the existing STFT kernel without
moving unrelated events.

Do not implement explicit transient/residual separation in this proof. That
branch requires a new multiresolution perfect-reconstruction split, adaptive
mask continuity, separate component processing, and recombination policy. It
also exposes threshold leakage and synthetic-component artifacts before the
smaller in-engine mechanism has been tested. Separation remains a research
fallback if the fixed-map peak proof fails its frozen gate.

## Clean-Room Rule

Public papers and public algorithm descriptions may inform Signal design.
Rubber Band source, unpublished R3 behavior, Elastique internals, and copied
implementation details are outside the research and implementation boundary.

## Next Task

Start Batch 29.6C with a report-only fixed-map peak transient proof. Add the
time-ramped analysis, peak-local group-delay and event collection state, then
run the unchanged current-grid gate. Keep adaptive resolution, linked stereo,
production routing, and transient/residual separation closed.
