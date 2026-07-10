# g10.029 Mono-Evidence Reassessment

Date: 2026-07-10
Status: structural design authorized; production promotion blocked

## Purpose

Decide whether the completed mono and objective evidence supports Batch 29.4
structural-hybrid planning, or whether all stretch work must pause for row-level
and independent stereo listening.

## Accepted Baseline

- centred offline analysis and exact output cropping preserve both endpoints
- all 60 broad-corpus rows pass length, endpoint-energy, added-silence, and
  peak-growth limits
- measured transient placement is effectively tied with Rubber Band across the
  matched corpus; no global timing defect is established
- the no-pitch-shift path shows no broad vocal-envelope failure
- Signal remains product-addressable as an OfflineHighQuality prototype, with
  the current production kernel and cache identity unchanged

## Structural Failure Targets

- broad identity locking causes the isolated `L001` transient crest spike;
  local phase-lock variants trade crest improvement against corpus timing and
  do not provide a production candidate
- long expansion sounds slightly grainier and measures excess fast spectral
  movement in `38/40` rows; added stationary ringing is not established
- fixed-ratio renders can end on a loud exterior sample, but additive,
  multiplicative, and centroid-selected fixed envelopes all failed listening
  or cross-source prediction
- linked-stereo peak, phase, and transient decisions remain undefined at the
  structural level; mid/side transport alone does not close that requirement

No measured fixed-ratio formant failure justifies a formant correction. Formant
policy remains a pitch-shift and independently reviewed stereo/vocal concern.

## Rejected Directions

- global removal of phase locking
- local scalar phase-lock selectors
- another one-parameter long-window probe
- source, additive-zero, multiplicative-zero, or centroid-selected endpoint
  envelopes
- a fixed-ratio formant correction without a measured failure

## Open Promotion Gates

- the 15-pair blind manifest is not row-complete
- stereo was not assessed and requires an independent listener
- linked-stereo behavior must cover the same policy as any successor kernel
- product-facing OfflineHighQuality receipts still require completed findings
  for all five real-source families

These gates block production replacement and Rubber Band-class claims. They do
not block design or report-only prototyping against already measured mono
failure targets.

## Decision

Open Batch 29.4 as a design-only structural checkpoint. It may define a
transient/tonal classifier, multiresolution ownership, deterministic
transitions, shared stereo decisions, and the first bounded report-only hybrid
candidate. It may not change the production default, cache identity, product
receipt posture, or RealtimePreview support flag.

The next design must treat the exterior-tail defect as an outcome for a
different algorithm class, not reopen fixed-envelope search. Candidate gates
must preserve the accepted baseline and report both the known local failures
and the candidate's corpus-wide worst cases.

## Next Task

Start the Batch 29.4 design tranche. Map current analysis, propagation,
synthesis, channel-link, and dynamic-ratio ownership. Then freeze the
transient/tonal classification, multiresolution window ownership, transition
law, shared stereo decisions, formant policy, validation matrix, and stop
conditions before implementing a hybrid candidate.
