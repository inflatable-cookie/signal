# 038 - Stretch Crate Surface And Evidence Consolidation

Status: planned; blocked on `g10.037`
Owner: dsp
Created: 2026-07-27
Depends on: `g10.036`, `g10.037`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `MAINTAINABILITY`

## Problem

`signal-dsp-stretch` carries `249` public items. Its two in-repo consumers,
`signal-render-plane` and `signal-runtime`, import roughly twenty. The audit
recorded seven structural findings behind that gap.

`A12` — the promotion policy is written three times.
`accepts_product_facing_path` encodes the rule set as a boolean,
`product_facing_path_blocker` encodes the same rules as reasons in a different
order, and `product_quality_rejection_note` encodes them a third time with
different wording. Three encodings of one gate can disagree.

`A13` — evidence scaffolding outweighs DSP roughly three to one, and duplicates
itself. `tonal_texture/spectral.rs` and `formant_boundary/spectral.rs` are
near-identical copies of `window_fits`, `hann_window`, and planner setup. Five
modules each construct their own `FftPlanner`.

`A14` — `transient_smear` exposes four public entry points wrapping one
eight-argument private function.

`A15` — `creative_cyclic::render` and `Plan::identity` are `#[cfg(test)]`.
Production calls only `render_continuous`, so tests exercise a path that ships
to nobody.

`A16` — `lib.rs` is `4855` lines, about `2300` of them tests. It holds tier
metadata, the RealtimePreview tier, three stretcher types, dynamic-ratio
segmentation, pitch composition, and the selector gates. The rest of the crate
is already well split; `lib.rs` never was.

`A6` — the offline engine calls `Fft::process` in `analyze_frame` and
`synthesize_frame`, which allocates scratch inside every call: two heap
allocations per STFT frame. The RealtimePreview kernel already uses
`process_with_scratch`.

`A7` — the expansion selector renders the input up to three times to make one
switch decision: the default output, a full draft baseline, then the
short-window render.

`A9` and `A10` are small: `run_phase_vocoder` computes an output offset from
`((prefix_frames - half_window) * ratio + half_window)` where both terms are
`window_size / 2`, so the ratio never participates and the result is always
`window_size / 2`; and two different `wrap_phase` implementations exist, with
the streaming path using the slower one.

## Generation Runway

This lane advances the `g10` runway from *correct and correctly identified*
to *small enough to keep correct*. It is the last lane before the offline
render architecture changes in `g10.039`, so it deliberately lands first and
gives that work a decomposed base.

The visible runway is:

1. surface inventory and retention decision
2. single-owner promotion gate and dead-surface removal
3. metric module consolidation
4. `lib.rs` decomposition
5. hot-loop and selector efficiency, byte-exact
6. closeout

The next planning checkpoint is Batch 38.1, which decides retention for every
public item and must not delete anything `g10.040` will need.

## Non-Goals

- no change to any rendered output; every batch here is byte-exact
- no RealtimePreview surface removal — `A11` belongs to `g10.040` and this lane
  leaves the RealtimePreview module intact apart from the one tautological test
  named in Batch 38.2
- no renderer, detector, selector-threshold, or algorithm change
- no cache, artifact, routing, runtime, Loophole, or Chorus work
- no new evidence metric

## Goals

- [ ] one owner per promotion rule
- [ ] one shared STFT surface for the evidence metrics
- [ ] one entry point per measurement with an explicit policy argument
- [ ] no production path that only exists under `cfg(test)`
- [ ] `lib.rs` decomposed to match the rest of the crate
- [ ] no per-frame heap allocation in the offline hot loop
- [ ] public surface reduced to what is used, retained deliberately, or
  scheduled by a named roadmap

## Execution Plan

### Batch 38.1 - Surface Inventory And Retention Decision

Status: blocked on `g10.037` Batch 37.4

Documentation only.

- [ ] list all `249` public items with consumer, test-only, or unused status
- [ ] mark every item retained, removed, or deferred to a named roadmap
- [ ] confirm the RealtimePreview surface is deferred to `g10.040` in full
- [ ] record the dead enum variants found by the audit —
  `IntegrationMode::CallbackSafeStreaming`,
  `CallbackTimelineMode::SourceProjected`,
  `UnsupportedMode::AudioThreadProcessing`,
  `UnsupportedMode::SourceAdvanceContract`,
  `UnsupportedMode::ChannelLayout`,
  `CallbackProcessError::CallbackProcessingUnsupported` — as `g10.040` inputs,
  not as removals here
- [ ] freeze byte-exactness as the acceptance proof for every later batch
- [ ] change documentation only

### Batch 38.2 - Promotion Gate And Dead Surface

Status: blocked on Batch 38.1

- [ ] reduce the promotion policy to one owner so the boolean form derives from
  the reason form
- [ ] prove the three previous encodings agreed, or record where they did not
- [ ] remove `fft_plans_ready`, which returns `Arc::strong_count(..) >= 1` and
  can never be false, and the assertion that depends on it
- [ ] remove the `cfg(test)`-only `creative_cyclic` render and identity paths,
  or make them production paths with a stated caller
- [ ] apply the Batch 38.1 removals

### Batch 38.3 - Metric Consolidation

Status: blocked on Batch 38.2

- [ ] extract one shared windowing and STFT surface for the evidence metrics
- [ ] collapse the four `transient_smear` entry points to one plus a policy
  argument
- [ ] prove every retained measurement returns identical values before and
  after

### Batch 38.4 - `lib.rs` Decomposition

Status: blocked on Batch 38.3

- [ ] split tier metadata, the stretcher types, dynamic-ratio segmentation,
  pitch composition, and the selector gates into modules matching the existing
  crate layout
- [ ] move the `lib.rs` test bodies to the modules they exercise
- [ ] remove the duplicate `wrap_phase` and the dead ratio term in
  `run_phase_vocoder`, with a comment recording what the offset actually is
- [ ] prove byte-exact output across the full regression surface

### Batch 38.5 - Hot Loop And Selector Efficiency

Status: blocked on Batch 38.4

- [ ] replace `Fft::process` with `process_with_scratch` in the offline engine
- [ ] remove the redundant renders in the expansion selector decision
- [ ] prove byte-exact output, and record before/after render time and peak
  heap for the corpus cases

### Batch 38.6 - Closeout

Status: blocked on Batch 38.5

- [ ] run `effigy validate` and the full crate suite
- [ ] publish the reduced public surface
- [ ] update the `g10` front doors
- [ ] name the next ready batch in `g10.039`

## Acceptance Criteria

- [ ] output is byte-exact against the `g10.036` baselines at every batch
- [ ] one promotion rule owner, with the boolean derived from the reason form
- [ ] no test asserts a condition that cannot fail
- [ ] no `cfg(test)`-only production path remains
- [ ] no module builds its own planner where a shared one exists
- [ ] `lib.rs` holds crate documentation, module wiring, and public re-exports
- [ ] zero per-frame heap allocations in the offline hot loop, proven
- [ ] every remaining public item has a consumer, a retention reason, or a
  named roadmap

## Risks and Mitigations

- Risk: consolidation changes a measurement and silently moves a gate.
  Mitigation: Batch 38.3 proves identical values for every retained metric
  before the old code is removed.
- Risk: deleting surface that `g10.040` needs. Mitigation: Batch 38.1 defers
  the whole RealtimePreview surface, and this lane touches only the one test
  helper that cannot fail.
- Risk: `process_with_scratch` changes results. Mitigation: it is the same
  transform with caller-owned scratch; byte-exactness is the acceptance proof,
  and the batch stops if it does not hold.
- Risk: decomposition churn collides with `g10.039`. Mitigation: this lane
  lands first by design and `g10.039` starts from the decomposed tree.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/`
- [ ] the Batch 38.1 public-surface inventory table
- [ ] byte-exactness proof per batch
- [ ] metric-equality proof for Batch 38.3
- [ ] render time and peak heap before/after for Batch 38.5
- [ ] commands actually run

## Next Task

Blocked. Open Batch 38.1 after `g10.037` Batch 37.4 closes.
