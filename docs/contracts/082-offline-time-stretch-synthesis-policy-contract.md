# 082 Offline Time-Stretch Synthesis Policy Contract

Status: active; report-only successor policy
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

### Rule 2: transient preservation is local time-map policy

The first proof uses the current `2048/512` synthesis grid and onset centres
from the frozen `g10.029` classifier. Every default-grid frame whose source
analysis window contains a detected onset belongs to that protected attack
island. Its synthesis centre is the exact projected onset plus its source-frame
offset from that onset, giving local ratio `1` across the overlapping frames.

Required duration compensation is distributed only across non-protected frames
between adjacent fixed anchors. Synthesis positions remain strictly monotonic.
Overlapping onset islands with incompatible synthesis positions are dense
conflicts. They must be reported and fall back to the current constant-hop path
for the conflicting interval.

### Rule 3: phase policy stays inside the engine

Protected attack frames reinitialize synthesis phase from their analysis phase.
Steady frames use instantaneous-frequency propagation and the current identity
locking policy. Entry and exit occur through overlapping frames on the same
grid; no time-domain transition crossfade is added.

### Rule 4: adaptive resolution is a later reconstruction stage

Short attack resolution and long stable-component resolution must share one
nonstationary frame schedule and compatible reconstruction weights. The
current independent `1024`, `2048`, and `4096` rendered branches remain
diagnostic controls only.

### Rule 5: exactness and evidence remain mandatory

Every proof retains identity bypass, centred boundary coverage, deterministic
output, finite samples, exact target length, and explicit mapping evidence.
The transient proof must report protected spans, compensation range, maximum
local hop, dense conflicts, anchor error, and current-versus-candidate quality.

### Rule 6: promotion stays closed

Production routing, cache identity, product receipts, pitch composition,
dynamic-ratio routing, RealtimePreview, and linked stereo remain unchanged.
Adaptive-resolution work opens only after the current-grid transient proof
passes its declared gate.

## Current-Grid Transient Proof Gate

- improve anchored `L001` crest by at least `3 dB`
- keep the candidate worst crest at or below `5.655483 dB`
- do not worsen corpus mean absolute event placement by more than `1` frame
- retain `60/60` integrity, transient, formant, boundary, and combined passes
- do not regress source-relative residual or unsupported-bin mass
- no non-finite output, non-monotonic synthesis position, uncovered output
  sample, or unreported dense-transient fallback

This gate proves only the transient/time-map mechanism. It does not waive the
fast spectral-movement gate required of the later combined mono candidate.

## Clean-Room Rule

Public papers and public algorithm descriptions may inform Signal design.
Rubber Band source, unpublished R3 behavior, Elastique internals, and copied
implementation details are outside the research and implementation boundary.

## Next Task

Implement the report-only current-grid adaptive transient timeline proof. Stop
if exact anchor placement and monotonic compensation cannot coexist.
