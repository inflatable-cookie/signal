# 082 Offline Time-Stretch Synthesis Policy Contract

Status: active; H/R/P separation passed, additive mono TSM rejected
Owner: dsp
Updated: 2026-07-10
Related contracts: `046`, `048`, `049`
Related architecture: `docs/architecture/offline-time-stretch-synthesis.md`

## Purpose

Freeze the synthesis-policy boundary after the first multi-output structural
hybrid failed its mono gate. This contract governs algorithm proof work. It
does not promote a product path.

## Rules

### Rule 1: components are additive, not alternative branches

The successor owns one monotonic source-to-output time map. Harmonic, residual,
and percussive components are a complementary decomposition of one source, not
alternative full-band renders. They may use specialized processors, but every
processor receives the same fixed ratio and exact target length. Final output
is their sample-aligned sum. No ownership crossfade, branch switching, delay
alignment, local time-map change, or component gain matching is allowed.

### Rule 2: separation is iterative H/R/P

The report-only separator accepts sample-rate metadata and performs two
centred, padded STFT decompositions:

1. Extract clearly harmonic bins from the input with a frame duration nearest
   `186 ms`, quarter-window hop, separation factor `beta_h=2`, and median spans
   nearest `200 ms` horizontally and `500 Hz` vertically.
2. Apply the same rule to the first-stage complement with a frame duration
   nearest `11.6 ms`, quarter-window hop, and `beta_p=2`. Clearly percussive
   bins become the percussive component. Everything else becomes residual.

Supported FFT sizes are powers of two. Magnitude-median boundaries replicate
the nearest valid frame or bin. Signal boundaries use the existing centred zero
padding and normalized overlap-add policy.

The binary masks are disjoint. At every time-frequency bin,
`M_h + M_r + M_p = 1`. Each masked spectrum retains the source complex phase.
No learned separator, soft-mask sweep, classifier guard, or post-separation
gain correction enters the first proof.

### Rule 3: separation must pass before component TSM

Batch 29.6D proves decomposition and reconstruction only. Harmonic, residual,
and percussive time-domain components must sum back to the exact source-domain
render within the declared numerical tolerance. No component is stretched in
this batch.

Synthetic controls must assign a steady bin-centred sinusoid primarily to the
harmonic component, an isolated broadband impulse primarily to the percussive
component, and stationary broadband noise primarily to the residual. The
expected owner must exceed either specialized non-owner by at least `12 dB`.
Failure rejects the separator before a corpus TSM render.

### Rule 4: component processing is fixed

Only after Batch 29.6D passes may Batch 29.6E apply:

- long-window identity-locked phase-vocoder TSM to the harmonic component
- the current `2048/512` OfflineHighQuality kernel to the residual component
- plain normalized OLA, using the short separation frame and quarter-window
  analysis hop, to the percussive component

OLA performs no waveform search, onset detection, transient reinsertion, phase
reset, or local timing compensation. Each component independently produces the
same target length from the same global ratio before sample-aligned addition.

### Rule 5: exactness and evidence remain mandatory

Every proof retains identity bypass, deterministic output, finite samples,
centred boundary coverage, exact target length, and explicit mapping evidence.
The separator reports mask population, partition error, component energy,
reconstruction RMS/peak error, endpoint error, and synthetic ownership. The
TSM proof adds component output lengths, component peak growth, transient
replica ratio, recombination peak growth, and current-versus-candidate quality.

### Rule 6: promotion stays closed

Production routing, cache identity, product receipts, pitch composition,
dynamic-ratio routing, RealtimePreview, and linked stereo remain unchanged.
Batch 29.6D passed on 2026-07-10. Batch 29.6E failed the frozen mono gate and
is rejected without tuning. Batch 29.7 remains closed.

## Separation Proof Gate

- masks are binary, mutually exclusive, and exhaustive for every analyzed bin
- component lengths equal input length
- recombined source peak error is at most `1e-5`
- recombined source RMS error is at most `1e-6`
- no non-finite component sample, uncovered source sample, or endpoint loss
- harmonic, percussive, and residual synthetic controls each meet the `12 dB`
  ownership margin
- identical input, sample rate, and parameters produce identical components

Batch 29.6D passes this gate. At `48 kHz`, the frozen geometry resolves to
`8192/2048` long analysis and `512/128` short analysis. The mixed reconstruction
control measured `8.940697e-8` peak error and `1.939046e-8` RMS error with zero
uncovered source samples. Ownership margins were `30.933980 dB` for the steady
sine, `164.871272 dB` for the isolated impulse, and `12.925746 dB` for stationary
noise. Repeated component vectors and hashes were identical.

## Additive Mono TSM Gate

- improve anchored `L001` crest by at least `3 dB`
- keep the candidate worst crest at or below `5.655483 dB`
- do not worsen corpus mean absolute event placement by more than `1` frame
- retain `60/60` integrity, transient, formant, boundary, and combined passes
- do not regress source-relative residual or unsupported-bin mass
- retain the original Batch 29.6 fast spectral-movement gate
- do not worsen the strongest post-attack secondary-peak/primary-peak ratio by
  more than `0.10` within one short percussive frame
- no non-finite output, non-monotonic synthesis position, uncovered output
  sample, component length mismatch, or hidden component gain correction

This gate proves the complete fixed-ratio additive mono mechanism. It does not
promote product routing or waive independent listening and linked-stereo gates.

## 2026-07-10 Additive H/R/P Proof Outcome

The additive candidate improved anchored `L001` crest by `3.375261 dB`, kept
worst crest to `4.083747 dB`, and reduced mean fast spectral movement at both
expansion ratios. It nevertheless failed the complete gate: measurable-row
mean event placement worsened `23.411637` frames, integrity passed `51/60`,
post-attack replica protection passed `26/48`, static residual and unsupported
bin mass regressed at both expansion ratios, and the combined gate passed
`0/60`.

Do not tune masks, separation factors, component gains, processor geometry, or
component timing. Do not open linked stereo.

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

## 2026-07-10 Fixed-Map Peak Proof Outcome

The fixed-map peak proof failed. Anchored `L001` crest improved only
`0.040942 dB`, measurable-row mean event placement worsened `16.851522`
frames, and the combined gate passed `12/60`. Integrity, added silence, peak
growth, and overlap-add coverage passed `60/60`, but `984/2370` guarded events
never reached a reported centre-threshold reset. Tonal residual regressed in
`21/60` rows and unsupported-bin mass regressed in `24/60`.

Do not tune the window-derived threshold, sensitivity, event guards, or reset
scope. Do not open adaptive resolution or linked stereo.

## 2026-07-10 H/R/P Reassessment Decision

The next proof uses refined harmonic/residual/percussive separation. The
residual component is mandatory: two-way H/P processing is known to route
ambiguous harmonic material such as voice into the short OLA path, where phase
jumps become audible. Iterative long/short separation and `beta=2` isolate only
clearly harmonic or percussive structures while preserving a complementary
residual.

This additive structure does not reopen the rejected full-band branch
crossfade. Component reconstruction is proven before TSM, and every component
uses the same output map and target length.

## Clean-Room Rule

Public papers and public algorithm descriptions may inform Signal design.
Rubber Band source, unpublished R3 behavior, Elastique internals, and copied
implementation details are outside the research and implementation boundary.

## Next Task

Stop component implementation and reassess the offline synthesis policy from
the measured Batch 29.6E failure. Decide whether a materially different
clean-room synthesis family warrants research before another card is opened.
Keep linked stereo, production routing, cache identity, pitch/dynamic,
RealtimePreview, and product integration closed.
