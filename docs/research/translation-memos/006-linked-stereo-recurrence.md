# Linked-Stereo Relationship-Preserving Recurrence

Status: Promoted
Memo: 006
Owner: dsp
Last updated: 2026-07-16
Related track: `g10.029`, Batch 29.7D

## 1) Project problem statement

Signal's first coherent linked-stereo proof shared schedule, geometry,
frequency traversal, and corrected/fallback decisions but advanced phase once
per channel. Mechanics passed. Quadrature phase, expansion delay, and
correlated-image controls failed by large margins. Two independent mono paths
reproduced every failure mask, assigning the defect to channel-local recurrence
rather than the shared decision layer.

The next topology must preserve current interchannel relationships without
mixing samples, flattening per-channel magnitudes, collapsing decorrelated
material, changing the frozen mono renderer, or inheriting GPL expression.

## 2) External evidence summary

Three independent primary sources converge on the same ownership shape.

- Signalsmith Stretch revision `57b93f4e` selects the channel with greatest
  prediction energy for each bin. It computes the vertical output phase only
  for that channel, then combines that output with each peer's current input
  complex relation to the reference. Each peer keeps its own target energy.
- Dorran, Lawlor, and Coyle's 2005 AES multichannel TSM paper states that the
  greater-magnitude same-bin peak is updated first and the lesser peak is then
  updated to preserve the original interchannel phase relationship. It also
  preserves magnitude differences by applying one schedule to both channels.
- Rubber Band R3 revision `e4296ac` identifies the greatest channel per bin.
  Inside its channel-lock range, a peer can borrow that channel's tracked peak
  trajectory while retaining a current analysis-phase offset. This is
  corroborating architecture only: Rubber Band is GPL and its control flow,
  ranges, scaling, constants, and expression are excluded.

The relationship law is also already represented independently in Signal's
rejected frequency-partitioned research prototype: owner output phase plus the
wrapped current input phase difference. That architecture was rejected for
full-band layer behavior. The cross-channel law was not isolated or rejected
by stereo evidence.

## 3) Recommendation

Replace per-channel phase recurrence in the two-channel report-only renderer
with reference-relative recurrence on every frame and bin:

1. Keep one shared fixed-ratio schedule, analysis geometry, traversal, crop,
   and overlap policy.
2. Select the reference channel from current target energy. Greater energy
   wins; exact ties choose the lower channel index.
3. Run the frozen coherent horizontal-plus-vertical recurrence once, for the
   reference channel only.
4. Preserve the reference result and its target magnitude.
5. For the peer, preserve its target magnitude and project the reference output
   through the current input complex ratio. In phase form:

   `output_peer_phase = output_ref_phase + wrap(input_peer_phase - input_ref_phase)`

6. An exactly silent peer remains exactly zero. If the reference recurrence
   fails the existing prediction-viability test, its existing current-input
   fallback makes the projected peer land on its own current input phase. Do
   not add a new threshold.
7. Keep accumulation and normalization per channel. No sample value from one
   channel may enter the other channel's magnitude or time-domain output.

This is relationship preservation, not dominant-channel phase replacement.
Only the reference recurrence is borrowed. The peer's current input phase
offset and magnitude remain explicit.

## 4) Tradeoffs the project would accept

- Per-bin reference ownership can switch between channels. The first proof
  measures switch stability instead of adding unevidenced hysteresis.
- Near the existing viability floor, exact phase relationships are less useful
  than deterministic finite fallback. The floor is not retuned for stereo.
- Two-channel fixed-ratio support remains report-only. More channels, dynamic
  ratio, realtime use, cache identity, and product routing remain closed.
- The topology is source-informed. Signal retains its own Rust structure,
  coherent predictor, thresholds, geometry, fallback, and validation.

## 5) What must be true before adoption

- frozen mono hashes do not change
- identity, duplicated mono, hard pan, channel swap, polarity, scaled duplicate,
  exact silence, coverage, boundaries, finiteness, and repeat still pass
- both channels become reference owners in deterministic controls
- an exact-energy tie and a controlled ownership crossing remain deterministic
  and do not create a boundary spike or image discontinuity
- the full frozen 29.7C quality gate passes unchanged at `0.75x`, `1.5x`, and
  `2.0x`
- no sample crossfeed or peer-magnitude borrowing occurs

## 6) Required prototype or validation work

Batch 29.7E is one report-only ablation. Replace only linked-stereo recurrence;
do not alter mono code, analysis representation, schedule, energy floor,
quality thresholds, or export policy.

Extend the mechanics report with per-channel reference counts, exact-tie
exercise, ownership-crossing exercise, maximum switch-boundary growth, and
repeat hashes. Then rerun the complete 29.7C phase, delay, image, decorrelated,
transient, replica, and crossfeed controls. Any mechanics or quality failure
stops before listening.

## 7) Promotion target

- `architecture work`
- `roadmap planning`

Promote into the offline synthesis architecture, contract `082` Rule 31H, and
`g10.029` Batch 29.7E.

## 8) Sources

| Source | Confidence | Notes |
| --- | --- | --- |
| [Signalsmith Stretch implementation](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h) | high | MIT source; greatest-energy per-bin reference and explicit current-input complex relation |
| [Multi-Channel Audio Time-scale Modification](https://mural.maynoothuniversity.ie/8793/1/BL-Multi-channel-2005.pdf) | high | AES paper; stronger same-bin peak first, weaker peak preserves original phase relationship |
| [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/PhaseAdvance.h) | high | GPL architecture evidence only; greatest-channel trajectory and analysis-relative offset |
| [Signal frequency-partitioned research prototype](../../../crates/signal-dsp-stretch/src/frequency_adaptive/source_studied.rs) | high | Local antecedent for the relation law; surrounding architecture remains rejected |

## Rejected Alternatives

### Shared output phase increment

Applying one reference phase increment to both prior output phases preserves a
previous output relationship. It does not explicitly restore the current input
relationship and can accumulate image error when delay or spatial phase moves.
The primary sources instead reconstruct the peer from a current analysis-phase
relation.

### Aggregate shared mode plus per-channel recurrence

This is the failed 29.7C topology. Independent mono attribution reproduces all
failure masks, so a different aggregate threshold or decision rule does not
address the primary defect.

### Mid/side synthesis or sample mixing

These change representation or introduce cross-channel sample ownership. They
are unnecessary to test the evidenced recurrence law and would obscure
crossfeed and mono-parity attribution.

### Rubber Band phase-lock translation

Rubber Band corroborates channel-linked trajectory ownership but its GPL
expression, guidance ranges, scaling, and constants are not transferable.

## Next Task

Batch 29.7E confirms the recurrence as a large improvement but not a gate pass.
29.7F excludes significant corrected-bin projection and real-edge constraint;
29.7G locates the first observable loss after support synthesis. 29.7H proves
analytic overlap is linearly equivalent and rejects it. 29.7I excludes the
remaining initial-frame, fallback, and weak-bin classes. Run Batch 29.7J to
calibrate the exact stereo invariant against ideal and external references
before changing recurrence or reopening listening.
